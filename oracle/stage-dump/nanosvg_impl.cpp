// oracle/stage-dump/nanosvg_impl.cpp
//
// liblibslic3r.a's NSVGUtils.cpp.o and svg.cpp.o members call nanosvg's nsvgParse,
// nsvgParseFromFile and nsvgDelete. Upstream provides their one implementation only from a GUI
// file (src/slic3r/GUI/BitmapCache.cpp), by defining NANOSVG_IMPLEMENTATION before including
// the header-only deps_src/nanosvg library -- a file this program never links, since the GUI is
// off. Static linking pulls in NSVGUtils.cpp.o and svg.cpp.o as whole units (for other symbols
// in the same members that this program's call graph does need), which makes their unresolved
// nsvg* calls a link error unless something, somewhere in the final link, defines them. This
// file is that something: the same one-time NANOSVG_IMPLEMENTATION include BitmapCache.cpp
// does, reached here instead of by editing anything under vendor/OrcaSlicer/. This program
// never calls into SVG embossing itself.

#define NANOSVG_IMPLEMENTATION
#include "nanosvg.h"
