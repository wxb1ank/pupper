# pupper

[![dependency status](https://deps.rs/crate/pupper/status.svg)](https://deps.rs/crate/pupper)
![docs.rs](https://img.shields.io/docsrs/pupper)

A Sony PlayStation 3 PUP (PlayStation Update Package) implementation.

## Overview

The PS3 receives software updates in a file format called 'PUP'. These packages are essentially
'flat' file systems: they contain individual files, or 'segments', but lack any hierarchical
structure.

This crate facilitates the creation and (de)serialization of PUPs.

## See Also

* <https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)>
