---
kind: intent
author: Bruk Habtu
---
# Intent: Port libslic3r to Rust with 1:1 functionality

## Problem

We want to port the slicer lib to Rust first, ensuring we have 1:1 functionality and using the old library as the oracle.

This is for me, and in the future for any other person that wants to build a whole slicer atop a more modern library.

The requester's suggestion for how: "We should leverage the same approach that was taken when the bun repo was migrated to rust. It worked out well." The exact workflow for porting, as per how the Bun Rust port worked, is to be sorted out as part of the spec.

What is ported is determined by what OrcaSlicer currently calls in the library. SLA is out.

The port is in a good place when it is usable to slice files for 3D printing. The requester uses Bambu printers.

Once the port is in a good place, the requester believes the next action is to ensure Rust idioms are in place, as well as optimizations. That is the following intent, not this one.

## Proposed outcome

- The slicer lib is ported to Rust.
- The Rust port has 1:1 functionality with the old library, apart from SLA.
- The Rust port produces the same output as the vendored slicer.
- As a first output, a benchy is sliced with the port and with the vendored slicer, and the results are compared.
- The port is usable to slice files for 3D printing.
- What is ported is what OrcaSlicer currently calls in the library.
- The old library is the oracle the port is checked against.
- SLA is not ported.
- Rust idioms and optimizations are not part of this work. They are the following intent.

## Affected users and systems

- The requester.
- In the future, any other person that wants to build a whole slicer atop a more modern library.
- The slicer lib in the vendored slicer, which is the old library and the oracle.
- Bambu printers, which the requester uses.

## Constraints

- The pin of the vendored slicer remains as it is.

## Open questions

### Q1: What is the exact workflow for porting, as per how the Bun Rust port worked?

This needs research into how the Bun Rust port was done.

### Q2: What counts as "the same output" as the vendored slicer, and how is it compared?

The requester's suggestion for a start: "Can we try slicing a benchy as a first output and compare the results." And: "the same can mean looking at the generated gcode."

### Q3: What does OrcaSlicer currently call in the library?

The answer determines what is ported.
