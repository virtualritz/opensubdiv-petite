#include <opensubdiv/far/stencilTable.h>
#include <opensubdiv/far/stencilTableFactory.h>
#include <opensubdiv/far/patchTable.h>
#include <vector>

#include "../vtr/types.hpp"

typedef OpenSubdiv::Far::LimitStencilTable LimitStencilTable;
typedef OpenSubdiv::Far::LimitStencilTableFactory LimitStencilTableFactory;
typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::TopologyRefiner TopologyRefiner;
typedef OpenSubdiv::Far::PatchTable PatchTable;

/// Flat FFI-safe replacement for C++ LocationArray.
struct LocationArrayDesc {
    int ptex_idx;
    int num_locations;
    const float* s;
    const float* t;
};

extern "C" {

void LimitStencilTable_destroy(const LimitStencilTable* table) {
    delete table;
}

/// Returns the 'u' derivative stencil interpolation weights.
FloatVectorRef LimitStencilTable_GetDuWeights(const LimitStencilTable* table) {
    auto& v = table->GetDuWeights();
    return FloatVectorRef(v.data(), v.size());
}

/// Returns the 'v' derivative stencil interpolation weights.
FloatVectorRef LimitStencilTable_GetDvWeights(const LimitStencilTable* table) {
    auto& v = table->GetDvWeights();
    return FloatVectorRef(v.data(), v.size());
}

/// Returns the 'uu' derivative stencil interpolation weights.
FloatVectorRef LimitStencilTable_GetDuuWeights(const LimitStencilTable* table) {
    auto& v = table->GetDuuWeights();
    return FloatVectorRef(v.data(), v.size());
}

/// Returns the 'uv' derivative stencil interpolation weights.
FloatVectorRef LimitStencilTable_GetDuvWeights(const LimitStencilTable* table) {
    auto& v = table->GetDuvWeights();
    return FloatVectorRef(v.data(), v.size());
}

/// Returns the 'vv' derivative stencil interpolation weights.
FloatVectorRef LimitStencilTable_GetDvvWeights(const LimitStencilTable* table) {
    auto& v = table->GetDvvWeights();
    return FloatVectorRef(v.data(), v.size());
}

/// Create a LimitStencilTable via the factory.
///
/// `options_bitfield` layout: [1:0] interpolationMode, [2] generate1stDerivatives,
/// [3] generate2ndDerivatives.
/// `fvar_channel` is passed separately.
///
/// `cv_stencils` and `patch_table` may be null.
const LimitStencilTable* LimitStencilTableFactory_Create(
    const TopologyRefiner* refiner,
    const LocationArrayDesc* location_descs,
    int num_arrays,
    const StencilTable* cv_stencils,
    const PatchTable* patch_table,
    unsigned int options_bitfield,
    unsigned int fvar_channel)
{
    // Rebuild the C++ LocationArrayVec from the flat descriptors.
    LimitStencilTableFactory::LocationArrayVec locations(num_arrays);
    for (int i = 0; i < num_arrays; ++i) {
        locations[i].ptexIdx      = location_descs[i].ptex_idx;
        locations[i].numLocations = location_descs[i].num_locations;
        locations[i].s            = location_descs[i].s;
        locations[i].t            = location_descs[i].t;
    }

    // Unpack bitfield into Options.
    LimitStencilTableFactory::Options options;
    options.interpolationMode      = options_bitfield & 0x3;
    options.generate1stDerivatives = (options_bitfield >> 2) & 0x1;
    options.generate2ndDerivatives = (options_bitfield >> 3) & 0x1;
    options.fvarChannel            = fvar_channel;

    return LimitStencilTableFactory::Create(
        *refiner, locations, cv_stencils, patch_table, options);
}

} // extern "C"
