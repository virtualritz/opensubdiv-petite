#pragma once

#include <opensubdiv/vtr/types.h>

struct IntVectorRef {
    const int* data;
    size_t sz;

    IntVectorRef(const int* data, size_t sz) : data(data), sz(sz) {}
};

struct FloatVectorRef {
    const float* data;
    size_t sz;

    FloatVectorRef(const float* data, size_t sz) : data(data), sz(sz) {}
};

struct IndexVectorRef {
    const OpenSubdiv::Vtr::Index* data;
    size_t sz;
    IndexVectorRef(const OpenSubdiv::Vtr::Index* data, size_t sz) : data(data), sz(sz) {}
};

