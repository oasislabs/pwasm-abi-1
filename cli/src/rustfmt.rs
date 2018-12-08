/// Copyright (c) 2013, Jyun-Yan You
///
/// All rights reserved.
///
/// Redistribution and use in source and binary forms, with or without
/// modification, are permitted provided that the following conditions
/// are met:
///
/// 1. Redistributions of source code must retain the above copyright
///    notice, this list of conditions and the following disclaimer.
/// 2. Redistributions in binary form must reproduce the above copyright
///    notice, this list of conditions and the following disclaimer in the
///    documentation and/or other materials provided with the distribution.
/// 3. Neither the name of the author nor the names of his contributors
///    may be used to endorse or promote products derived from this software
/// without specific prior written permission.
///
/// THIS SOFTWARE IS PROVIDED BY THE AUTHORS "AS IS" AND
/// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
/// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
/// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHORS OR CONTRIBUTORS BE LIABLE
/// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
/// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
/// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
/// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
/// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
/// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
/// SUCH DAMAGE.
use std::io::{self, Write};
use std::process;
use std::sync::{Once, ONCE_INIT};

// This function was copied from the following file:
// https://github.com/rust-lang/rust-bindgen/blob/c3955ddeac236f4a6b967d5b8ddc8dc21238d152/tests/tests.rs
//
// Run `rustfmt` on the given source string and return a tuple of the formatted
// bindings, and rustfmt's stderr.
pub fn format(source: String) -> (String, String) {
    static CHECK_RUSTFMT: Once = ONCE_INIT;

    CHECK_RUSTFMT.call_once(|| {
        let have_working_rustfmt = process::Command::new("rustup")
            .args(&["run", "nightly", "rustfmt", "--version"])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .ok()
            .map_or(false, |status| status.success());

        if !have_working_rustfmt {
            panic!(
                "
The latest `rustfmt` is required to run the `bindgen` test suite. Install
`rustfmt` with:
    $ rustup update nightly
    $ rustup run nightly cargo install -f rustfmt-nightly
"
            );
        }
    });

    let mut child = process::Command::new("rustup")
        .args(&["run", "nightly", "rustfmt"])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("should spawn `rustup run nightly rustfmt`");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || stdin.write_all(source.as_bytes()));

    // Read stderr on a new thread for similar reasons.
    let stderr_handle = ::std::thread::spawn(move || {
        let mut output = vec![];
        io::copy(&mut stderr, &mut output).map(|_| String::from_utf8_lossy(&output).to_string())
    });

    let mut output = vec![];
    io::copy(&mut stdout, &mut output).expect("Should copy stdout into vec OK");

    // Ignore actual rustfmt status because it is often non-zero for trivial
    // things.
    let _ = child.wait().expect("should wait on rustfmt child OK");

    stdin_handle
        .join()
        .expect("writer thread should not have panicked")
        .expect("should have written to child rustfmt's stdin OK");

    let bindings = String::from_utf8(output).expect("rustfmt should only emit valid utf-8");

    let stderr = stderr_handle
        .join()
        .expect("stderr reader thread should not have panicked")
        .expect("should have read child rustfmt's stderr OK");

    (bindings, stderr)
}
