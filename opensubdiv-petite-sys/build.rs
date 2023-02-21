#![allow(unreachable_code)]
use std::{env, path::PathBuf};

//#[cfg(all(target_os = "macos", feature = "openmp"))]
//static MAC_OS_BREW_CLANG_PATH: &str = "/usr/local/opt/llvm";

pub fn main() {
    #[cfg(all(target_os = "macos", feature = "cuda"))]
    panic!("The feature `cuda` is not available on macOS.");

    #[cfg(all(not(target_os = "macos"), feature = "metal"))]
    panic!("The feature `metal` is only available on macOS.");

    // FIXME: check if Homebrew Clang is installed in
    // /usr/local/Cellar/llvm/11.1.0/bin

    let mut open_subdiv = cmake::Config::new("OpenSubdiv");

    open_subdiv
        .always_configure(true)
        .define("NO_EXAMPLES", "1")
        .define("NO_TUTORIALS", "1")
        .define("NO_REGRESSION", "1")
        .define("NO_PTEX", "1")
        .define("NO_DOC", "1")
        .define("NO_OPENCL", "1")
        .define("NO_CLEW", "1")
        .define("NO_TBB", "1")
        .define("NO_GLFW", "1");

    #[cfg(any(target_os = "macos", not(feature = "cuda")))]
    open_subdiv.define("NO_CUDA", "1");

    #[cfg(any(target_os = "macos", not(feature = "openmp")))]
    open_subdiv.define("NO_OMP", "1");

    #[cfg(any(not(target_os = "macos"), not(feature = "metal")))]
    open_subdiv.define("NO_METAL", "1");

    /* FIXME: OMP support on macOS
    #[cfg(all(target_os = "macos", feature = "openmp"))]
    {
        // We try to use Homebrew's Clang for building; not Apple Clang.
        // This allows us to build with OpenMP support.
        let clang_path = PathBuf::from(MAC_OS_BREW_CLANG_PATH);
        let clang = clang_path.join("bin").join("clang");
        let clang_pp = clang_path.join("bin").join("clang++");
        let clang_lib = clang_path.join("lib");

        if clang.exists() && clang_pp.exists() {
            open_subdiv
                .define("CMAKE_C_COMPILER", clang)
                .define("CMAKE_CXX_COMPILER", clang_pp)
                .define("OPENMP_LIBRARIES", &clang_lib)
                .define("OPENMP_INCLUDES", clang_path.join("include"));

            println!("cargo:rustc-link-search=native={}", clang_lib.display());
        } else {
            // No clang installed via Homebrew – we can't build with OpenMP
            // support on macOS as Apple's Clang has no support for it.
            panic!("Feature `openmp` enabled but no OpenMP capable compiler found.")
        }

    }*/

    let open_subdiv = open_subdiv.build();

    let osd_inlude_path = open_subdiv.join("include");
    let osd_lib_path = open_subdiv.join("lib");

    let mut osd_capi = cc::Build::new();

    osd_capi
        .include(&osd_inlude_path)
        .cpp(true)
        .static_flag(true)
        .flag("-std=c++14")
        .flag("-Wno-return-type-c-linkage")
        .file("c-api/far/primvar_refiner.cpp")
        .file("c-api/far/stencil_table.cpp")
        .file("c-api/far/stencil_table_factory.cpp")
        .file("c-api/far/topology_refiner.cpp")
        .file("c-api/far/topology_level.cpp")
        .file("c-api/osd/cpu_evaluator.cpp")
        .file("c-api/osd/cpu_vertex_buffer.cpp");

    #[cfg(all(feature = "openmp", not(target_os = "macos")))]
    osd_capi
        .file("c-api/osd/omp_evaluator.cpp")
        .file("c-api/osd/omp_vertex_buffer.cpp");

    #[cfg(all(feature = "cuda", not(target_os = "macos")))]
    osd_capi
        .include(&osd_inlude_path)
        .file("c-api/osd/cuda_evaluator.cpp")
        .file("c-api/osd/cuda_vertex_buffer.cpp");

    osd_capi.compile("osd-capi");

    println!("cargo:rustc-link-lib=static=osd-capi");

    println!("cargo:rustc-link-search=native={}", osd_lib_path.display());
    println!("cargo:rustc-link-lib=static=osdCPU");

    #[cfg(all(feature = "openmp", not(target_os = "macos")))]
    println!("cargo:rustc-link-lib=static=osdOMP");

    #[cfg(feature = "cuda")]
    println!("cargo:rustc-link-lib=static=osdGPU");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");

    println!("cargo:rerun-if-changed=wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg("-IOpenSubdiv")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_type("OpenSubdiv.*")
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)
        .layout_tests(false);

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bindings = bindings
        .clang_args(&["-F", osd_inlude_path.to_str().unwrap()])
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-changed=build.rs");
}
