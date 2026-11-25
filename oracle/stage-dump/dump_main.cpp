// oracle/stage-dump/dump_main.cpp
//
// R5 / plan step 8's dumper. Drives the vendored, locally built libslic3r through the sequence
// vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp uses (read by this task rather than
// linked, since test_helpers belongs to the fff_print_tests target, not to libslic3r):
//
//   Model model = Model::read_from_file(<fixture>);
//   for (ModelObject *mo : model.objects) { mo->ensure_on_bed(); print.auto_assign_extruders(mo); }
//   print.apply(model, config);
//   print.validate();
//   print.process();
//   print.export_gcode(<out>/plate_1.gcode, &result, nullptr);
//
// and registers Print::set_slicing_pipeline_hook_fn so dump_seams.hpp's one entry point fires
// once per seam R5 enumerates. This program never calls an arranger (R24 and R10 fix placement
// in the fixture) and never forces gcode_comments: it applies exactly the configuration it is
// given, plus the one arming key below.
//
// Run with the repository root as the working directory, as `just oracle-dump` runs it: it
// shells out to `oracle/provenance` by that relative path for every dump's header, and reads
// docs/ORACLE.md by the same relative path in --set mode.
//
// Two configuration-driver modes (C12's decision, see oracle/stage-dump/README.md). --set is the
// default: with neither flag given, a run still takes the --set driver with no extra override of
// its own, which is what makes the bare `stage_dump <fixture> <configuration> --out <dir>` a
// valid run.
//   --config <path>   the oracle's resolved configuration (an --export-settings JSON). Applied
//                      over DynamicPrintConfig::full_print_config() and changed in nothing but
//                      the arming key.
//   --set k=v ...     repeatable (zero or more), together with the docs/ORACLE.md row named by
//                      <configuration>'s own Overrides column. Resolves no preset and no
//                      inherits chain (R11).
//
// --unarmed skips hook registration and the arming key; `just oracle-dump` uses it for the L1
// identity check's second run, over the same fixture, configuration and driver arguments.

#include "libslic3r/Config.hpp"
#include "libslic3r/GCode/GCodeProcessor.hpp"
#include "libslic3r/GCode/GCodeStatistics.hpp"
#include "libslic3r/Model.hpp"
#include "libslic3r/Print.hpp"
#include "libslic3r/PrintConfig.hpp"
#include "libslic3r_version.h"

#include "dump_seams.hpp"

#include <algorithm>
#include <array>
#include <cctype>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <system_error>
#include <vector>

namespace fs = std::filesystem;
using namespace Slic3r;

namespace {

// ---------------------------------------------------------------------------
// argv parsing.
// ---------------------------------------------------------------------------

struct Args {
    std::string fixture;
    std::string configuration;
    std::string out_dir;
    std::string config_path;              // --config <path>
    std::vector<std::string> set_overrides; // --set <key>=<value>, repeatable
    bool unarmed = false;                   // --unarmed
};

[[noreturn]] void fail(const std::string &msg) {
    std::cerr << "error: " << msg << "\n";
    std::exit(1);
}

[[noreturn]] void usage_error(const std::string &msg) {
    std::cerr << "error: " << msg << "\n"
              << "usage: stage_dump <fixture> <configuration> --out <dir> "
                 "[--config <path> | --set <key>=<value> ...] [--unarmed]\n";
    std::exit(1);
}

Args parse_args(int argc, char **argv) {
    Args args;
    std::vector<std::string> positional;
    for (int i = 1; i < argc; ++i) {
        std::string a = argv[i];
        if (a == "--out") {
            if (++i >= argc)
                usage_error("--out needs a value");
            args.out_dir = argv[i];
        } else if (a == "--config") {
            if (++i >= argc)
                usage_error("--config needs a value");
            args.config_path = argv[i];
        } else if (a == "--set") {
            if (++i >= argc)
                usage_error("--set needs a value");
            args.set_overrides.push_back(argv[i]);
        } else if (a == "--unarmed") {
            args.unarmed = true;
        } else if (!a.empty() && a[0] == '-') {
            usage_error("unknown option " + a);
        } else {
            positional.push_back(a);
        }
    }
    if (positional.size() != 2)
        usage_error("expected exactly two positional arguments, <fixture> <configuration>");
    args.fixture = positional[0];
    args.configuration = positional[1];
    if (args.out_dir.empty())
        usage_error("--out is required");
    if (!args.config_path.empty() && !args.set_overrides.empty())
        usage_error("--config cannot be combined with --set");
    // Neither flag is required: with no --config and no --set, the run still takes the --set
    // driver (full_print_config() plus the row's own Overrides, which may themselves be
    // "(none)") -- this is what makes the bare `just oracle-dump <fixture> <configuration>`,
    // with no further arguments, a valid run.
    return args;
}

// ---------------------------------------------------------------------------
// Small string / markdown-table helpers (deliberately duplicated from oracle/provenance's own
// table reader: these are two standalone programs in two different languages, and neither
// imports the other).
// ---------------------------------------------------------------------------

std::string trim(const std::string &s) {
    size_t b = s.find_first_not_of(" \t\r\n");
    size_t e = s.find_last_not_of(" \t\r\n");
    return b == std::string::npos ? std::string() : s.substr(b, e - b + 1);
}

std::vector<std::string> split_cells(const std::string &line) {
    std::string s = trim(line);
    if (!s.empty() && s.front() == '|')
        s.erase(0, 1);
    if (!s.empty() && s.back() == '|')
        s.pop_back();
    std::vector<std::string> cells;
    std::stringstream ss(s);
    std::string cell;
    while (std::getline(ss, cell, '|'))
        cells.push_back(trim(cell));
    return cells;
}

std::string strip_cell(std::string cell) {
    cell = trim(cell);
    if (cell.size() >= 2 && cell.front() == '`' && cell.back() == '`')
        cell = trim(cell.substr(1, cell.size() - 2));
    std::string lower = cell;
    std::transform(lower.begin(), lower.end(), lower.begin(),
                    [](unsigned char c) { return static_cast<char>(std::tolower(c)); });
    if (lower.empty() || lower == "(none)" || lower == "-" || lower == "\xe2\x80\x94")
        return std::string();
    return cell;
}

std::string shell_quote(const std::string &s) {
    std::string out = "'";
    for (char c : s) {
        if (c == '\'')
            out += "'\\''";
        else
            out += c;
    }
    out += "'";
    return out;
}

// Runs `cmd` through the shell, capturing stdout ONLY -- the child's stderr is left attached to
// this program's own stderr, so a warning it prints (oracle/provenance warns on stderr when
// docs/ORACLE.md has no "Built oracle flags" section yet) is visible on the terminal but never
// captured into the string this function returns. That matters here because the return value
// becomes a dump's leading header: a stray non-";"-prefixed line in it would make
// oracle/provenance --check's own leading_comment_block reader find no header at all. Exits the
// whole program, naming `cmd`, on a non-zero exit: every dump's header comes from a call to
// oracle/provenance (never formatted locally), so going on without one would be worse than
// stopping.
std::string run_capture(const std::string &cmd) {
    FILE *pipe = popen(cmd.c_str(), "r");
    if (!pipe)
        fail("failed to run: " + cmd);
    std::string output;
    std::array<char, 4096> buf{};
    size_t n;
    while ((n = fread(buf.data(), 1, buf.size(), pipe)) > 0)
        output.append(buf.data(), n);
    int status = pclose(pipe);
    if (status != 0)
        fail("command failed (" + cmd + "); see its stderr above");
    return output;
}

// oracle/slice's own fixture-resolution rule (task 11-fixture-set): an argument with no path
// separator and no extension is a bare stem, resolved against fixtures/ by appending .stl;
// anything else (a path, or a name that already carries an extension) is used as given.
std::string resolve_fixture_path(const std::string &fixture) {
    if (fixture.find('/') == std::string::npos && fs::path(fixture).extension().empty())
        return "fixtures/" + fixture + ".stl";
    return fixture;
}

// oracle/provenance's --readme table is keyed by the fixture's frozen basename (e.g.
// "20mm_cube.stl"), not by the bare stem `just oracle-dump` is invoked with -- so this program
// resolves and passes the basename of resolve_fixture_path's result, never the raw argument.
std::string provenance_header(const std::string &fixture, const std::string &configuration) {
    const std::string basename = fs::path(resolve_fixture_path(fixture)).filename().string();
    return run_capture("oracle/provenance " + shell_quote(basename) + " " +
                        shell_quote(configuration) + " --oracle built --syntax gcode");
}

void apply_override(DynamicPrintConfig &config, const std::string &item) {
    auto pos = item.find('=');
    if (pos == std::string::npos)
        usage_error("override '" + item + "' is not key=value");
    const std::string key = item.substr(0, pos);
    const std::string value = item.substr(pos + 1);
    try {
        config.set_deserialize_strict(key, value);
    } catch (const std::exception &exc) {
        fail("could not apply override '" + item + "': " + exc.what());
    }
}

// Reads docs/ORACLE.md's configuration row named `configuration` and returns its Overrides
// cell's ;-separated, trimmed, non-empty parts. --set mode's own reading of that table: never
// the Machine/Process/Filament(s) columns, which --set mode never resolves (R11 puts preset
// resolution out of scope).
std::vector<std::string> read_row_overrides(const std::string &configuration) {
    std::ifstream in("docs/ORACLE.md");
    if (!in)
        fail("docs/ORACLE.md not found (run stage_dump from the repository root)");
    std::string line;
    std::vector<std::string> header;
    bool have_header = false;
    while (std::getline(in, line)) {
        const std::string t = trim(line);
        if (t.empty() || t.front() != '|') {
            have_header = false;
            continue;
        }
        std::vector<std::string> cells = split_cells(line);
        if (!have_header) {
            header = cells;
            have_header = true;
            continue;
        }
        if (header.empty() || header[0] != "Configuration")
            continue;
        if (!cells.empty() && strip_cell(cells[0]) == configuration) {
            int idx = -1;
            for (size_t i = 0; i < header.size(); ++i) {
                if (header[i] == "Overrides") {
                    idx = static_cast<int>(i);
                    break;
                }
            }
            if (idx < 0 || static_cast<size_t>(idx) >= cells.size())
                return {};
            std::vector<std::string> parts;
            std::stringstream ss(cells[static_cast<size_t>(idx)]);
            std::string part;
            while (std::getline(ss, part, ';')) {
                std::string v = strip_cell(part);
                if (!v.empty())
                    parts.push_back(v);
            }
            return parts;
        }
    }
    fail("no configuration row named '" + configuration + "' in docs/ORACLE.md");
}

// Builds the config this run applies: DynamicPrintConfig::full_print_config() as the base in
// both driver modes (never a preset), then either the --config JSON or the row's plus the
// command line's --set overrides, then the arming key unless --unarmed.
DynamicPrintConfig build_config(const Args &args) {
    DynamicPrintConfig config = DynamicPrintConfig::full_print_config();
    if (!args.config_path.empty()) {
        // Apply it and change nothing but the arming key.
        ConfigSubstitutionContext ctxt(ForwardCompatibilitySubstitutionRule::Disable);
        std::map<std::string, std::string> key_values;
        std::string reason;
        const int rc = config.load_from_json(args.config_path, ctxt,
                                              /*load_inherits_to_config=*/true, key_values, reason);
        if (rc != 0)
            fail("failed to load --config " + args.config_path + ": " + reason);
    } else {
        for (const std::string &ov : read_row_overrides(args.configuration))
            apply_override(config, ov);
        for (const std::string &ov : args.set_overrides)
            apply_override(config, ov);
    }
    if (!args.unarmed)
        config.set_key_value("slicing_pipeline_plugin", new ConfigOptionStrings({"probe"}));
    return config;
}

} // namespace

int main(int argc, char **argv) {
    Args args = parse_args(argc, argv);

    std::error_code ec;
    fs::create_directories(args.out_dir, ec);
    if (ec)
        fail("could not create --out directory " + args.out_dir + ": " + ec.message());

    const std::string fixture_path = resolve_fixture_path(args.fixture);
    if (!fs::exists(fixture_path))
        fail("no such fixture file: " + fixture_path);

    DynamicPrintConfig config = build_config(args);

    size_t dump_count = 0;
    std::string header;
    if (!args.unarmed) {
        header = provenance_header(args.fixture, args.configuration);
        Print::set_slicing_pipeline_hook_fn(
            [&](Print &print, const PrintObject *object, SlicingPipelineStepPlugin step) {
                oracle_stage_dump::dump_seam(args.out_dir, header, print, object, step);
                ++dump_count;
            });
    } else {
        Print::set_slicing_pipeline_hook_fn(nullptr);
    }

    Print print;
    Model model = Model::read_from_file(fixture_path);
    for (ModelObject *mo : model.objects) {
        mo->ensure_on_bed();
        print.auto_assign_extruders(mo);
    }
    print.apply(model, config);

    // Set is_BBL_printer before validation, exactly as the CLI does
    // (src/OrcaSlicer.cpp:7011-7026): print.validate()'s relative-extruder-addressing check
    // (Print.cpp:2795-2800) is gated on !is_BBL_printer(), and Print::apply never derives the
    // flag itself -- the CLI sets it from the resolved config's printer_model after loading a
    // preset bundle. is_BBL_printer() is a Print-level flag the CLI itself sets from outside the
    // config, never a config key, so setting it here does not touch "the configuration it is
    // given" (the one rule --config mode holds to) in either driver mode:
    //  - --config mode derives it from the loaded config's own printer_model, the same field the
    //    CLI reads, keeping a run comparable to the release oracle's for whatever printer the
    //    loaded resolved configuration names;
    //  - --set mode never learns a printer_model at all (R11: it resolves no preset, including
    //    no Machine column), so it is set true directly -- every row of docs/ORACLE.md's
    //    configuration table builds a Bambu printer's print, and this mode constructs the
    //    configurations that table's Machine column would otherwise supply, not a different
    //    printer family.
    if (args.config_path.empty())
        print.is_BBL_printer() = true;
    else
        print.is_BBL_printer() = (config.opt_string("printer_model", true).compare(0, 9, "Bambu Lab") == 0);

    const StringObjectException err = print.validate();
    if (!err.string.empty()) {
        Print::set_slicing_pipeline_hook_fn(nullptr);
        fail("print.validate() failed: " + err.string);
    }

    print.process();
    // Reset immediately after process(), the point by which every seam for this run has fired:
    // the setter holds a static, and leaving it registered would leak the hook into anything
    // else linked in the same process (upstream's own reasoning,
    // test_slicing_pipeline_hook.cpp:30-36).
    Print::set_slicing_pipeline_hook_fn(nullptr);

    const std::string gcode_path = args.out_dir + "/plate_1.gcode";
    GCodeProcessorResult result;
    print.export_gcode(gcode_path, &result, nullptr);

    const std::string settings_path = args.out_dir + "/settings.json";
    config.save_to_json(settings_path, "project_settings", "project", SoftFever_VERSION);

    try {
        export_gcode_statistics(result, gcode_statistics_path(gcode_path));
    } catch (const std::exception &exc) {
        fail(std::string("export_gcode_statistics failed: ") + exc.what());
    }

    if (!args.unarmed)
        std::cout << "dumps: " << dump_count << "\n";

    return 0;
}
