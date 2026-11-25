// oracle/stage-dump/dump_seams.cpp
//
// Created by this task (25-stage-dumper), holding the seam entry point and no per-seam state
// reading. Part 2 (26-stage-dump-seams) fills dump_seam's body with the state each seam leaves
// behind; here it writes only the provenance header plus the seam's name and the object's
// identity, which is what the L1 identity check and the hook-fires-at-all proof need.

#include "dump_seams.hpp"

#include "libslic3r/Print.hpp"

#include <cstdlib>
#include <fstream>
#include <iostream>

namespace oracle_stage_dump {

using Slic3r::Print;
using Slic3r::PrintObject;
using Slic3r::SlicingPipelineStepPlugin;

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

} // namespace

void dump_seam(const std::string &out_dir, const std::string &provenance_header, Print &print,
               const PrintObject *object, SlicingPipelineStepPlugin step) {
    const std::string path = out_dir + "/" + seam_name(step) + ".dump";
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
    // Part 2 (26-stage-dump-seams) extends this body to serialise the state the seam leaves
    // behind (Layer::lslices, LayerRegion::slices, raw_slices, perimeters, thin_fills,
    // fill_surfaces, fills, PrintObject::support_layers(), object_skirt()), as integer coord_t
    // coordinates never round-tripped through float. This task's dump carries only the two
    // lines above.
    if (!out) {
        std::cerr << "error: write failed for " << path << "\n";
        std::exit(1);
    }
}

} // namespace oracle_stage_dump
