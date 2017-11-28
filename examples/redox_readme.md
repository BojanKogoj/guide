
**Redox** is an operating system written in Rust, a language with focus on safety and high performance. Redox, following the microkernel design, aims to be secure, usable, and free. Redox is inspired by previous kernels and operating systems, such as SeL4, MINIX, Plan 9, and BSD.

Redox _is not_ just a kernel, it's a **full-featured Operating System**, providing packages (memory allocator, file system, display manager, core utilities, etc.) that together make up a functional and convenient operating system. You can loosely think of it as the GNU or BSD ecosystem, but in a memory safe language and with modern technology. See [this list](#ecosystem) for overview of the ecosystem.

The website can be found at https://www.redox-os.org.

Please make sure you use the **latest nightly** of `rustc` before building (for more troubleshooting, see ["Help! Redox won't compile!"](#compile-help)).

[![Travis Build Status](https://travis-ci.org/redox-os/redox.svg?branch=master)](https://travis-ci.org/redox-os/redox)
[![Downloads](https://img.shields.io/github/downloads/redox-os/redox/total.svg)](https://github.com/redox-os/redox/releases)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)
![Rust Version](https://img.shields.io/badge/rust-nightly%202017--10--03-lightgrey.svg)

## Contents

* [What it looks like](#screenshots)
* [Ecosystem](#ecosystem)
* [Help! Redox won't compile](#compile-help)
* [Contributing to Redox](#contributing)
* [Cloning, Building and running](#cloning-building-running)
 * [Quick Setup](#quick-setup)
 * [Manual Setup](#manual-setup)
 * [Setup Using Docker](#setup-using-docker)
