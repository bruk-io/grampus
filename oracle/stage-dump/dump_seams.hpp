#pragma once

// oracle/stage-dump/dump_seams.hpp
//
// The one entry point dump_main.cpp's slicing-pipeline hook calls, once per fired seam (R5,
// plan step 8). This task (25-stage-dumper) builds the program around this entry point and
// fills it with only the provenance header plus the seam's name and the object's identity --
// enough to prove the dumping build is harmless (the L1 identity check in just oracle-dump)
// and that the hook fires at all. Part 2, 26-stage-dump-seams, extends the body to also
// serialise the state each seam leaves behind (Layer::lslices, LayerRegion::slices,
// raw_slices, perimeters, thin_fills, fill_surfaces, fills, PrintObject::support_layers(),
// object_skirt()), per R5 and plan step 8. This task reads none of that state.

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
// oracle/provenance, --syntax gcode -- this function never formats one itself) followed by two
// lines, the seam's name and the object's identity -- its index in print.objects(), or "print"
// when `object` is null, for one of the two print-wide seams -- and nothing else in this task.
// Exits the whole program on a write failure, naming the path: a dumper that silently dropped a
// dump would be worse than one that stops.
void dump_seam(const std::string &out_dir, const std::string &provenance_header,
               Slic3r::Print &print, const Slic3r::PrintObject *object,
               Slic3r::SlicingPipelineStepPlugin step);

} // namespace oracle_stage_dump
