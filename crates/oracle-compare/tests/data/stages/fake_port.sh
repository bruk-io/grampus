#!/bin/sh
# Committed stand-in for `grampus-cli` (task `39-compare-stages`'s own tests): a real executable
# `crates/oracle-compare/src/stages.rs`'s `run_port_binary` can actually spawn and whose output it
# actually reads, unlike `/usr/bin/true` (which always writes nothing, so no test could ever drive
# `run()`'s dump-compare or fallback branches end to end). Parses the same two flags
# `run_port_binary` passes -- `--out <dir>` and `--dump-stages <dir>` -- and, only when
# `STAGE_DUMP_SOURCE` names a directory, copies every `*.dump` file in it into the `--dump-stages`
# directory and, if it holds a `plate_1.gcode`, copies that into the `--out` directory too. With
# `STAGE_DUMP_SOURCE` unset (the default), this writes nothing at all, the same as `/usr/bin/true`
# did -- so the existing "not ported" / "missing oracle dump" tests, which never set it, see no
# behaviour change. Always exits 0: the real `grampus-cli` exits non-zero until every call in its
# five-call `Surface` sequence succeeds, but no test here depends on this stand-in's own exit
# code, only on what it writes.
set -eu

out_dir=""
dump_dir=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out)
      out_dir="$2"
      shift 2
      ;;
    --dump-stages)
      dump_dir="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -n "${STAGE_DUMP_SOURCE:-}" ] && [ -d "$STAGE_DUMP_SOURCE" ]; then
  if [ -n "$dump_dir" ]; then
    mkdir -p "$dump_dir"
    for f in "$STAGE_DUMP_SOURCE"/*.dump; do
      [ -e "$f" ] && cp "$f" "$dump_dir"/
    done
  fi
  if [ -n "$out_dir" ] && [ -f "$STAGE_DUMP_SOURCE/plate_1.gcode" ]; then
    mkdir -p "$out_dir"
    cp "$STAGE_DUMP_SOURCE/plate_1.gcode" "$out_dir"/
  fi
fi

exit 0
