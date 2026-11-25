#pragma once

// oracle/stage-dump/dump_seams.hpp
//
// The one entry point dump_main.cpp's slicing-pipeline hook calls, once per fired seam (R5,
// plan step 8). Task 25-stage-dumper built the program around this entry point and gave it only
// the provenance header plus the seam's name and the object's identity -- enough to prove the
// dumping build is harmless (the L1 identity check in just oracle-dump) and that the hook fires
// at all. This task, 26-stage-dump-seams, fills dump_seam's body with the state each of R5's
// eleven seams leaves behind: per object, Layer::lslices and its print_z/slice_z/height, each
// LayerRegion's slices/raw_slices/thin_fills/fill_surfaces/perimeters/fills, and each
// SupportLayer's support_islands/support_fills; for the two print-wide seams, Print::skirt(),
// Print::skirt_brim_groups(), PrintObject::object_skirt() and Print::wipe_tower_data()'s scalar
// geometry fields. See README.md's "Dump format" section for the exact record shape, the
// floating-point form, and the ordering rule; this file states only the entry point's contract,
// not the body it writes.

#include <string>

namespace Slic3r {
class Print;
class PrintObject;
enum class SlicingPipelineStepPlugin;
} // namespace Slic3r

namespace oracle_stage_dump {

// The seam's name, exactly as SlicingPipelineStepPlugin's enumerator is spelled (e.g.
// "posSlice", "psWipeTower"). This is also the dump's filename stem: 05-oracle-provenance
// fixes the committed path as port/dumps/<fixture-stem>-<configuration>/<seam>.dump, one file
// per fired seam, since every committed fixture holds exactly one object and so the seam name
// alone is unique within a run.
const char *seam_name(Slic3r::SlicingPipelineStepPlugin step);

// Writes <out_dir>/<seam_name(step)>.dump: `provenance_header` (already formatted by a call to
// oracle/provenance, --syntax gcode -- this function never formats one itself), then `; seam:
// <name>` and `; object: <index|print>`, then the seam's own body -- the state the step it
// follows has produced, in the record shape README.md's "Dump format" section documents.
// Exits the whole program on a write failure, naming the path: a dumper that silently dropped a
// dump would be worse than one that stops.
void dump_seam(const std::string &out_dir, const std::string &provenance_header,
               Slic3r::Print &print, const Slic3r::PrintObject *object,
               Slic3r::SlicingPipelineStepPlugin step);

} // namespace oracle_stage_dump
