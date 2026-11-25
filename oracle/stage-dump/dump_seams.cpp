// oracle/stage-dump/dump_seams.cpp
//
// R5 / plan step 8's per-seam serialisers. Task 25-stage-dumper built the seam entry point,
// the provenance header and the object-identity line; this task (26-stage-dump-seams) fills in
// the body each serialiser writes -- the state the step it follows has produced. See R5 in
// intent/libslic3r-rust-port/spec.md for the eleven seams and why every other pipeline step has
// none; this file's comments name only the seam each function handles and do not restate R5.
//
// Record format (documented in full in README.md's "Dump format" section): every line after
// the header is self-describing -- "field=<name> layer=<L|-> region=<R|-> ..." plus whatever
// extra keys that field needs (object=, group=, brim=, filament_id=, surface=, poly=, path=,
// ring=) -- so a reader finds a value without depending on file order. "-" is the sentinel for
// "not applicable" (e.g. a print-wide field's layer, or a field with no per-region structure).
// Integer coord_t coordinates are written as plain decimal integers, never through unscale,
// unscale_, unscaled or any other float/double conversion. A field that is genuinely
// floating-point in the library (print_z, slice_z, height, the wipe tower's scalar fields) is
// written with "%a" (a hexadecimal floating-point literal, exact for the double or float it
// holds -- promoted to double for the formatter, never narrowed) and carries no decimal form.
//
// Every per-object seam (posSlice through posSimplifyPath) calls the same dump_object_state:
// the hook fires once per step with no record of what changed, and each of these nine steps
// only ever *adds to or refines* the same growing state (Layer::lslices and its print_z/
// slice_z/height; per LayerRegion, slices, raw_slices, thin_fills, fill_surfaces, perimeters,
// fills; per SupportLayer, support_islands and support_fills) -- a field a later step has not
// reached yet is simply still empty, which is the true state at that seam, not a fabricated
// one. Each seam still gets its own named function, so a reviewer greps the enumerator name
// and finds one, and each function's own one-line comment says which step it follows.

#include "dump_seams.hpp"

#include "libslic3r/ExPolygon.hpp"
#include "libslic3r/ExtrusionEntity.hpp"
#include "libslic3r/ExtrusionEntityCollection.hpp"
#include "libslic3r/Layer.hpp"
#include "libslic3r/Point.hpp"
#include "libslic3r/Polygon.hpp"
#include "libslic3r/Print.hpp"
#include "libslic3r/Surface.hpp"
#include "libslic3r/SurfaceCollection.hpp"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <set>
#include <string>

namespace oracle_stage_dump {

using Slic3r::ExPolygon;
using Slic3r::ExPolygons;
using Slic3r::ExtrusionEntitiesPtr;
using Slic3r::ExtrusionEntity;
using Slic3r::ExtrusionEntityCollection;
using Slic3r::ExtrusionLoop;
using Slic3r::ExtrusionMultiPath;
using Slic3r::ExtrusionPath;
using Slic3r::Layer;
using Slic3r::LayerRegion;
using Slic3r::Point;
using Slic3r::Point3;
using Slic3r::Points;
using Slic3r::Print;
using Slic3r::PrintObject;
using Slic3r::SlicingPipelineStepPlugin;
using Slic3r::SupportLayer;
using Slic3r::Surfaces;
using Slic3r::WipeTowerData;

const char *seam_name(SlicingPipelineStepPlugin step) {
    switch (step) {
    case SlicingPipelineStepPlugin::posSlice:
        return "posSlice";
    case SlicingPipelineStepPlugin::posPerimeters:
        return "posPerimeters";
    case SlicingPipelineStepPlugin::posEstimateCurledExtrusions:
        return "posEstimateCurledExtrusions";
    case SlicingPipelineStepPlugin::posPrepareInfill:
        return "posPrepareInfill";
    case SlicingPipelineStepPlugin::posInfill:
        return "posInfill";
    case SlicingPipelineStepPlugin::posIroning:
        return "posIroning";
    case SlicingPipelineStepPlugin::posContouring:
        return "posContouring";
    case SlicingPipelineStepPlugin::posSupportMaterial:
        return "posSupportMaterial";
    case SlicingPipelineStepPlugin::posDetectOverhangsForLift:
        return "posDetectOverhangsForLift";
    case SlicingPipelineStepPlugin::posSimplifyPath:
        return "posSimplifyPath";
    case SlicingPipelineStepPlugin::psWipeTower:
        return "psWipeTower";
    case SlicingPipelineStepPlugin::psSkirtBrim:
        return "psSkirtBrim";
    case SlicingPipelineStepPlugin::psGCodePostProcess:
        return "psGCodePostProcess";
    }
    return "unknown-seam";
}

namespace {

// The object's index in print.objects(), or -1 if not found (should not happen: the hook only
// ever passes either nullptr or a pointer drawn from that same list).
int object_index(const Print &print, const PrintObject *object) {
    int idx = 0;
    for (const auto *candidate : print.objects()) {
        if (candidate == object)
            return idx;
        ++idx;
    }
    return -1;
}

// ---------------------------------------------------------------------------
// The "did not fire" roster (part of this task's own "Done when": `just oracle-dump` names
// every seam that did not fire). The roster is R5's eleven seams plus
// posEstimateCurledExtrusions, which R5 itself says to report as not fired rather than as
// missing (its fire site can never satisfy hook_after's is_step_done check for a sliced
// object). psGCodePostProcess is deliberately absent: R5 excludes it by name -- it fires from
// the GUI G-code post-process seam, never from Print::process(), so this program could never
// observe it firing or not firing, and listing it here would misreport "unreachable from this
// program" as "did not fire this run".
//
// Populated from dump_seam() below, which only ever runs on an armed invocation, and reported
// once at process exit via std::atexit. Registration is itself lazy (on the first dump_seam()
// call), not unconditional: `just oracle-dump` runs this same binary a second time with
// --unarmed for the L1 identity check, and that run never calls dump_seam() at all -- an
// unconditional atexit would print a "did not fire" roster for a run where arming, and so
// every seam, was deliberately off.
const std::array<const char *, 12> kSeamRoster = {
    "posSlice",
    "posPerimeters",
    "posEstimateCurledExtrusions",
    "posPrepareInfill",
    "posInfill",
    "posIroning",
    "posContouring",
    "posSupportMaterial",
    "posDetectOverhangsForLift",
    "posSimplifyPath",
    "psWipeTower",
    "psSkirtBrim",
};

std::set<std::string> &fired_seams() {
    static std::set<std::string> fired;
    return fired;
}

void report_unfired_seams() {
    for (const char *name : kSeamRoster)
        if (fired_seams().count(name) == 0)
            std::cout << "did not fire: " << name << "\n";
}

// ---------------------------------------------------------------------------
// Small formatting helpers shared by every serialiser below. Every argument here is read by
// value or by const reference; nothing in this file takes a non-const reference into the
// Print or the PrintObject the hook handed over, so none of it can mutate the pipeline it is
// recording.
// ---------------------------------------------------------------------------

// Exact for the double (or float, promoted to double by the varargs call) it is given: never
// narrowed, never routed through a decimal conversion that could lose a bit. README.md's
// "Dump format" section names this as the one floating-point form this file ever writes.
std::string hexfloat(double value) {
    char buf[64];
    std::snprintf(buf, sizeof buf, "%a", value);
    return std::string(buf);
}

// Writes `key=value` with a leading space, so a call site is a flat chain of these.
void kv(std::ostream &out, const char *key, long long value) { out << ' ' << key << '=' << value; }
void kv(std::ostream &out, const char *key, const std::string &value) { out << ' ' << key << '=' << value; }

// `index < 0` writes the "not applicable" sentinel "-" instead of a number -- used for a
// print-wide field's layer/region, and for support_layers(), which has no region axis at all.
void kv_idx(std::ostream &out, const char *key, long index) {
    out << ' ' << key << '=';
    if (index < 0)
        out << '-';
    else
        out << index;
}

// coord_t is int64_t at this pin (libslic3r.h); written as a plain decimal integer.
void write_point(std::ostream &out, const Point &p) {
    out << static_cast<long long>(p.x()) << ',' << static_cast<long long>(p.y());
}
void write_point3(std::ostream &out, const Point3 &p) {
    out << static_cast<long long>(p.x()) << ',' << static_cast<long long>(p.y()) << ','
        << static_cast<long long>(p.z());
}

void write_points(std::ostream &out, const Points &pts) {
    for (size_t i = 0; i < pts.size(); ++i) {
        if (i)
            out << ' ';
        write_point(out, pts[i]);
    }
}

// Writes the common "field=... layer=... region=...<ctx>" prefix every record below shares.
void write_prefix(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                   const std::string &ctx) {
    out << "field=" << field;
    kv_idx(out, "layer", layer_idx);
    kv_idx(out, "region", region_idx);
    out << ctx;
}

// Writes one line per ring (the contour, then each hole, in their stored order -- never
// sorted) of a single polygon-shaped record, self-describing with `index_key` (e.g. "poly" for
// a plain ExPolygons field, "surface" for a SurfaceCollection's own index) plus `index`, and
// `type_tag` (already formatted as " type=stTop", or empty) for a SurfaceCollection's surface
// type -- not a coordinate, so it is read off the enum by name rather than through hexfloat.
void write_expolygon_rings(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                            const std::string &ctx, const char *index_key, long index,
                            const std::string &type_tag, const ExPolygon &ex) {
    auto ring_line = [&](const std::string &ring, const Points &pts) {
        write_prefix(out, field, layer_idx, region_idx, ctx);
        kv(out, index_key, index);
        out << type_tag;
        kv(out, "ring", ring);
        kv(out, "n", static_cast<long long>(pts.size()));
        out << " pts=";
        write_points(out, pts);
        out << "\n";
    };
    ring_line("contour", ex.contour.points);
    for (size_t h = 0; h < ex.holes.size(); ++h)
        ring_line("hole:" + std::to_string(h), ex.holes[h].points);
}

// A plain ExPolygons field (Layer::lslices, LayerRegion::raw_slices, SupportLayer::
// support_islands): one summary line naming the field's polygon count, then one line per ring
// of each polygon, in `polys`' own stored order -- never sorted, never deduplicated.
void write_expolygons_field(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                             const std::string &ctx, const ExPolygons &polys) {
    write_prefix(out, field, layer_idx, region_idx, ctx);
    kv(out, "count", static_cast<long long>(polys.size()));
    out << "\n";
    for (size_t p = 0; p < polys.size(); ++p)
        write_expolygon_rings(out, field, layer_idx, region_idx, ctx, "poly", static_cast<long>(p), "",
                               polys[p]);
}

const char *surface_type_name(Slic3r::SurfaceType type) {
    switch (type) {
    case Slic3r::stTop:
        return "stTop";
    case Slic3r::stBottom:
        return "stBottom";
    case Slic3r::stBottomBridge:
        return "stBottomBridge";
    case Slic3r::stInternalAfterExternalBridge:
        return "stInternalAfterExternalBridge";
    case Slic3r::stInternal:
        return "stInternal";
    case Slic3r::stInternalSolid:
        return "stInternalSolid";
    case Slic3r::stInternalBridge:
        return "stInternalBridge";
    case Slic3r::stSecondInternalBridge:
        return "stSecondInternalBridge";
    case Slic3r::stInternalVoid:
        return "stInternalVoid";
    case Slic3r::stPerimeter:
        return "stPerimeter";
    case Slic3r::stCount:
        break;
    }
    return "unknown-surface-type";
}

// A SurfaceCollection field (LayerRegion::slices, LayerRegion::fill_surfaces): the same shape
// as write_expolygons_field, except each surface's own ring lines also carry its SurfaceType.
void write_surfaces_field(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                           const std::string &ctx, const Surfaces &surfaces) {
    write_prefix(out, field, layer_idx, region_idx, ctx);
    kv(out, "count", static_cast<long long>(surfaces.size()));
    out << "\n";
    for (size_t s = 0; s < surfaces.size(); ++s) {
        const std::string type_tag = std::string(" type=") + surface_type_name(surfaces[s].surface_type);
        write_expolygon_rings(out, field, layer_idx, region_idx, ctx, "surface", static_cast<long>(s),
                               type_tag, surfaces[s].expolygon);
    }
}

// A single leaf extrusion path's own record: its polyline's points (Polyline3 / Point3, three
// integer coordinates per point -- this program never reads ExtrusionPath::width/height/
// mm3_per_mm/overhang_degree/smooth_speed, since R5 names the containing collections
// (perimeters, fills, thin_fills, support_fills) and not their per-path flow metadata; see
// README.md's Decisions section). `loop` records whether this path's nearest ancestor was an
// ExtrusionLoop (closed) rather than a bare ExtrusionPath or an ExtrusionMultiPath (open) -- a
// fact about the path's own shape, not a coordinate, so it is written as 0/1, never hex.
void write_path_record(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                        const std::string &ctx, size_t path_idx, const ExtrusionPath &path, bool loop) {
    write_prefix(out, field, layer_idx, region_idx, ctx);
    kv(out, "path", static_cast<long long>(path_idx));
    kv(out, "role", ExtrusionEntity::role_to_string(path.role()));
    kv(out, "loop", loop ? 1LL : 0LL);
    kv(out, "n", static_cast<long long>(path.polyline.points.size()));
    out << " pts=";
    for (size_t i = 0; i < path.polyline.points.size(); ++i) {
        if (i)
            out << ' ';
        write_point3(out, path.polyline.points[i]);
    }
    out << "\n";
}

// Counts the leaf extrusion paths a collection's own walk would visit, with no output -- used
// to size write_extrusion_field's summary line before the detail lines are written. Mirrors
// walk_entity_paths' own descent exactly (same recursion, same three leaf kinds), so the two
// can never disagree on what counts as a leaf.
size_t count_entity_paths(const ExtrusionEntitiesPtr &entities) {
    size_t total = 0;
    for (const ExtrusionEntity *e : entities) {
        if (e->is_collection()) {
            const auto *coll = dynamic_cast<const ExtrusionEntityCollection *>(e);
            if (coll)
                total += count_entity_paths(coll->entities);
        } else if (e->is_loop()) {
            const auto *loop = dynamic_cast<const ExtrusionLoop *>(e);
            if (loop)
                total += loop->paths.size();
        } else if (const auto *multi = dynamic_cast<const ExtrusionMultiPath *>(e)) {
            total += multi->paths.size();
        } else if (dynamic_cast<const ExtrusionPath *>(e)) {
            total += 1;
        }
    }
    return total;
}

// Walks one ExtrusionEntityCollection's own entities in their stored order -- never sorted,
// never deduplicated -- descending into a nested collection in place (so a grouping
// collection contributes no record of its own, only its leaves), and writing one record per
// leaf ExtrusionPath: a bare ExtrusionPath directly, or each of an ExtrusionLoop's or an
// ExtrusionMultiPath's own `paths` in turn (both hold an ExtrusionPaths vector; the loop/
// multipath object itself owns no geometry beyond that vector, so its own first/last-point
// bookkeeping is not duplicated here). `path_idx` is a running counter over the whole
// top-level collection, assigned in this same walk order, so it names a path without
// depending on which container held it.
void walk_entity_paths(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                        const std::string &ctx, const ExtrusionEntitiesPtr &entities, size_t &path_idx) {
    for (const ExtrusionEntity *e : entities) {
        if (e->is_collection()) {
            const auto *coll = dynamic_cast<const ExtrusionEntityCollection *>(e);
            if (coll)
                walk_entity_paths(out, field, layer_idx, region_idx, ctx, coll->entities, path_idx);
        } else if (e->is_loop()) {
            const auto *loop = dynamic_cast<const ExtrusionLoop *>(e);
            if (loop)
                for (const ExtrusionPath &p : loop->paths)
                    write_path_record(out, field, layer_idx, region_idx, ctx, path_idx++, p, true);
        } else if (const auto *multi = dynamic_cast<const ExtrusionMultiPath *>(e)) {
            for (const ExtrusionPath &p : multi->paths)
                write_path_record(out, field, layer_idx, region_idx, ctx, path_idx++, p, false);
        } else if (const auto *p = dynamic_cast<const ExtrusionPath *>(e)) {
            write_path_record(out, field, layer_idx, region_idx, ctx, path_idx++, *p, false);
        }
        // No other ExtrusionEntity subclass exists in the vendored tree at the pin; an
        // entity matching none of the above is silently skipped rather than guessed at.
    }
}

// An ExtrusionEntityCollection field (LayerRegion::thin_fills/perimeters/fills, SupportLayer::
// support_fills, Print::skirt(), PrintObject::object_skirt(), a SkirtBrimGroup's own skirt or
// brim): one summary line naming the field's total leaf-path count, then one record per leaf
// path in walk_entity_paths' own order. Always written, even when `collection` is empty --
// psSkirtBrim's object_skirt() record exists for every print object this way, with count=0
// and no path lines following, which is still "one object_skirt() record" a reviewer can point
// to by name.
void write_extrusion_field(std::ofstream &out, const char *field, long layer_idx, long region_idx,
                            const std::string &ctx, const ExtrusionEntityCollection &collection) {
    write_prefix(out, field, layer_idx, region_idx, ctx);
    kv(out, "count", static_cast<long long>(count_entity_paths(collection.entities)));
    out << "\n";
    size_t path_idx = 0;
    walk_entity_paths(out, field, layer_idx, region_idx, ctx, collection.entities, path_idx);
}

// A single scalar floating-point field (a layer's print_z/slice_z/height, one of the wipe
// tower's scalar fields): one line, no layer/region prefix is written for a wipe-tower field
// (layer_idx < 0), and `layer` is the owning layer's index for the three per-layer ones.
void write_double_field(std::ofstream &out, const char *field, long layer_idx, double value) {
    out << "field=" << field;
    kv_idx(out, "layer", layer_idx);
    kv(out, "value", hexfloat(value));
    out << "\n";
}

void write_int_field(std::ofstream &out, const char *field, long long value) {
    out << "field=" << field;
    kv(out, "value", value);
    out << "\n";
}

// ---------------------------------------------------------------------------
// Per-object state (R5's nine per-object seams all call this; see this file's own top comment
// for why one function suffices for all nine). Walks object->layers() and, per region,
// object->regions(); then object->support_layers().
// ---------------------------------------------------------------------------

void dump_object_state(std::ofstream &out, const PrintObject *object) {
    const auto layers = object->layers();
    for (size_t li = 0; li < layers.size(); ++li) {
        const Layer *layer = layers[li];
        const long layer_idx = static_cast<long>(li);
        write_double_field(out, "print_z", layer_idx, layer->print_z);
        write_double_field(out, "slice_z", layer_idx, layer->slice_z);
        write_double_field(out, "height", layer_idx, layer->height);
        write_expolygons_field(out, "lslices", layer_idx, -1, "", layer->lslices);

        for (size_t ri = 0; ri < layer->regions().size(); ++ri) {
            const LayerRegion *region = layer->get_region(static_cast<int>(ri));
            const long region_idx = static_cast<long>(ri);
            write_surfaces_field(out, "slices", layer_idx, region_idx, "", region->slices.surfaces);
            write_expolygons_field(out, "raw_slices", layer_idx, region_idx, "", region->raw_slices);
            write_extrusion_field(out, "thin_fills", layer_idx, region_idx, "", region->thin_fills);
            write_surfaces_field(out, "fill_surfaces", layer_idx, region_idx, "",
                                  region->fill_surfaces.surfaces);
            write_extrusion_field(out, "perimeters", layer_idx, region_idx, "", region->perimeters);
            write_extrusion_field(out, "fills", layer_idx, region_idx, "", region->fills);
        }
    }

    const auto support_layers = object->support_layers();
    for (size_t si = 0; si < support_layers.size(); ++si) {
        const SupportLayer *layer = support_layers[si];
        const std::string ctx = " support_layer=" + std::to_string(si);
        write_expolygons_field(out, "support_islands", -1, -1, ctx, layer->support_islands);
        write_extrusion_field(out, "support_fills", -1, -1, ctx, layer->support_fills);
    }
}

} // namespace

// ---------------------------------------------------------------------------
// The nine per-object serialisers. Each is a one-line wrapper so a reviewer greps the
// enumerator name and finds a function that names it; dump_object_state's own comment (above)
// is where the shared reasoning lives.
// ---------------------------------------------------------------------------

namespace {

// posSlice: the step that produces Layer::lslices and each layer's print_z/slice_z/height.
void dump_posSlice(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posPerimeters: the step that produces LayerRegion::slices (detect_surfaces_type), raw_slices
// (the pre-hook backup) and perimeters, plus thin_fills (generated by the perimeter
// generator).
void dump_posPerimeters(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posEstimateCurledExtrusions is never relied on (R5): no serialiser-specific state to add,
// and dump_seam never calls this function in practice (see the roster above) -- this program
// still carries no case for it that would claim otherwise.
// posPrepareInfill: the step that produces LayerRegion::fill_surfaces.
void dump_posPrepareInfill(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posInfill: the step that produces LayerRegion::fills.
void dump_posInfill(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posIroning: refines LayerRegion::fills (adds ironing paths) and fill_surfaces; no field of
// its own beyond the shared state dump_object_state already walks.
void dump_posIroning(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posContouring: Z-contouring (belt printers only; does not fire for a non-belt fixture, and
// R5 does not name a field of its own for it beyond the lslices geometry it may reshape).
void dump_posContouring(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posSupportMaterial: the step that produces SupportLayer::support_islands and support_fills.
void dump_posSupportMaterial(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }
// posDetectOverhangsForLift: marks existing perimeter paths rather than producing a field of
// its own; dump_object_state's shared walk still reflects every change already made.
void dump_posDetectOverhangsForLift(std::ofstream &out, const PrintObject *object) {
    dump_object_state(out, object);
}
// posSimplifyPath: simplifies the point count of existing perimeters/fills paths in place; no
// field of its own, the shared walk reflects the simplified geometry.
void dump_posSimplifyPath(std::ofstream &out, const PrintObject *object) { dump_object_state(out, object); }

// psWipeTower: Print::wipe_tower_data(). Called with the default filaments_cnt=0, which
// Print::wipe_tower_data (Print.cpp) only ever estimates from when psWipeTower is NOT yet
// done; by the time this hook fires, Print::process() has already called set_done(psWipeTower)
// (Print.cpp, just above the hook's own fire site), so the early-return branch always taken
// here is the `is_step_done(psWipeTower)` one, which returns the already-computed
// m_wipe_tower_data with no mutation -- confirmed by reading that function before writing
// this call, so the L1 identity check this program's own recipe runs is never put at risk by
// a read that turns out not to be one. Only the scalar geometry fields are written
// (depth/width/height/brim_width/bbx/rib_offset/number_of_toolchanges); priming, tool_changes
// and final_purge carry emitted G-code in unscaled mm and are not read here -- see
// README.md's Decisions section.
void dump_psWipeTower(std::ofstream &out, const Print &print) {
    const WipeTowerData &wtd = print.wipe_tower_data();
    write_double_field(out, "wipe_tower_depth", -1, wtd.depth);
    write_double_field(out, "wipe_tower_width", -1, wtd.width);
    write_double_field(out, "wipe_tower_height", -1, wtd.height);
    write_double_field(out, "wipe_tower_brim_width", -1, wtd.brim_width);
    write_int_field(out, "wipe_tower_number_of_toolchanges",
                    static_cast<long long>(wtd.number_of_toolchanges));
    out << "field=wipe_tower_bbx";
    kv(out, "defined", wtd.bbx.defined ? 1LL : 0LL);
    kv(out, "min", hexfloat(wtd.bbx.min.x()) + "," + hexfloat(wtd.bbx.min.y()));
    kv(out, "max", hexfloat(wtd.bbx.max.x()) + "," + hexfloat(wtd.bbx.max.y()));
    out << "\n";
    out << "field=wipe_tower_rib_offset";
    kv(out, "value", hexfloat(double(wtd.rib_offset.x())) + "," + hexfloat(double(wtd.rib_offset.y())));
    out << "\n";
}

// psSkirtBrim: Print::skirt(), Print::skirt_brim_groups() and, per the task's own "the state
// each seam reads" section, PrintObject::object_skirt() for every print object -- separate
// state from Print::skirt(), not a view of it (see dump_seams.hpp's and this task's task.md's
// own account of PrintObject::m_skirt). One object_skirt() record is written per print object
// even when its own collection is empty, so the Done-when line asking for one such record per
// object is satisfied literally, not just for the configurations where per-object skirts
// happen to be in use.
void dump_psSkirtBrim(std::ofstream &out, const Print &print) {
    write_extrusion_field(out, "skirt", -1, -1, "", print.skirt());

    const auto &groups = print.skirt_brim_groups();
    // A summary line for the vector itself, always written (even when it is empty, as it is
    // for a configuration with no skirt and no brim), so "Print::skirt_brim_groups()" is a
    // field a reviewer can point to in the dump by name -- the per-group/per-brim lines below
    // exist only when a group actually has one.
    out << "field=skirt_brim_groups";
    kv_idx(out, "layer", -1);
    kv_idx(out, "region", -1);
    kv(out, "count", static_cast<long long>(groups.size()));
    out << "\n";
    for (size_t g = 0; g < groups.size(); ++g) {
        const std::string group_ctx = " group=" + std::to_string(g);
        write_extrusion_field(out, "skirt_brim_group_skirt", -1, -1, group_ctx, groups[g].skirt);
        for (size_t b = 0; b < groups[g].brims.size(); ++b) {
            const std::string brim_ctx =
                group_ctx + " brim=" + std::to_string(b) +
                " filament_id=" + std::to_string(groups[g].brims[b].filament_id);
            write_extrusion_field(out, "skirt_brim_group_brim", -1, -1, brim_ctx, groups[g].brims[b].brim);
        }
    }

    const auto objects = print.objects();
    for (size_t o = 0; o < objects.size(); ++o) {
        const std::string object_ctx = " object=" + std::to_string(o);
        write_extrusion_field(out, "object_skirt", -1, -1, object_ctx, objects[o]->object_skirt());
    }
}

} // namespace

void dump_seam(const std::string &out_dir, const std::string &provenance_header, Print &print,
               const PrintObject *object, SlicingPipelineStepPlugin step) {
    const std::string path = out_dir + "/" + seam_name(step) + ".dump";
    // The filename is the seam name alone, with no object-index suffix (every committed
    // fixture holds exactly one object, so it is unique within a run today -- see
    // README.md's "Decisions"). A second object firing the same seam would silently
    // overwrite the first object's dump instead of being caught, so refuse to proceed
    // rather than lose one silently: a multi-object fixture has to fail loudly here
    // until a later task gives the name an object component.
    if (std::filesystem::exists(path)) {
        std::cerr << "error: " << path
                   << " already exists in this run -- a second object fired the same seam, "
                      "which the single-object filename this task uses cannot name "
                      "separately (see dump_seams.cpp's dump_seam)\n";
        std::exit(1);
    }
    std::ofstream out(path, std::ios::out | std::ios::trunc | std::ios::binary);
    if (!out) {
        std::cerr << "error: could not write " << path << "\n";
        std::exit(1);
    }
    out << provenance_header;
    out << "; seam: " << seam_name(step) << "\n";
    if (object == nullptr) {
        out << "; object: print\n";
    } else {
        out << "; object: " << object_index(print, object) << "\n";
    }

    switch (step) {
    case SlicingPipelineStepPlugin::posSlice:
        dump_posSlice(out, object);
        break;
    case SlicingPipelineStepPlugin::posPerimeters:
        dump_posPerimeters(out, object);
        break;
    case SlicingPipelineStepPlugin::posEstimateCurledExtrusions:
        // Never actually reached (see the roster comment above); no body to write.
        break;
    case SlicingPipelineStepPlugin::posPrepareInfill:
        dump_posPrepareInfill(out, object);
        break;
    case SlicingPipelineStepPlugin::posInfill:
        dump_posInfill(out, object);
        break;
    case SlicingPipelineStepPlugin::posIroning:
        dump_posIroning(out, object);
        break;
    case SlicingPipelineStepPlugin::posContouring:
        dump_posContouring(out, object);
        break;
    case SlicingPipelineStepPlugin::posSupportMaterial:
        dump_posSupportMaterial(out, object);
        break;
    case SlicingPipelineStepPlugin::posDetectOverhangsForLift:
        dump_posDetectOverhangsForLift(out, object);
        break;
    case SlicingPipelineStepPlugin::posSimplifyPath:
        dump_posSimplifyPath(out, object);
        break;
    case SlicingPipelineStepPlugin::psWipeTower:
        dump_psWipeTower(out, print);
        break;
    case SlicingPipelineStepPlugin::psSkirtBrim:
        dump_psSkirtBrim(out, print);
        break;
    case SlicingPipelineStepPlugin::psGCodePostProcess:
        // Excluded by R5 (fires from the GUI seam, not from here); never reached.
        break;
    }

    if (!out) {
        std::cerr << "error: write failed for " << path << "\n";
        std::exit(1);
    }

    fired_seams().insert(seam_name(step));
    static bool atexit_registered = false;
    if (!atexit_registered) {
        std::atexit(report_unfired_seams);
        atexit_registered = true;
    }
}

} // namespace oracle_stage_dump
