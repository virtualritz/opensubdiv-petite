#include <opensubdiv/far/patchTable.h>
#include <opensubdiv/far/patchTableFactory.h>
#include <opensubdiv/far/topologyRefiner.h>
#include <opensubdiv/far/types.h>
#include <opensubdiv/far/patchDescriptor.h>
#include <opensubdiv/far/patchParam.h>

typedef OpenSubdiv::Far::PatchTable PatchTable;
typedef OpenSubdiv::Far::TopologyRefiner TopologyRefiner;
typedef OpenSubdiv::Far::PatchTableFactory::Options Options;
typedef OpenSubdiv::Far::PatchDescriptor PatchDescriptor;
typedef OpenSubdiv::Far::PatchParam PatchParam;
typedef OpenSubdiv::Far::ConstIndexArray ConstIndexArray;

extern "C" {

// PatchTableFactory functions
PatchTable* PatchTableFactory_Create(TopologyRefiner* refiner, Options* options) {
    if (options) {
        return OpenSubdiv::Far::PatchTableFactory::Create(*refiner, *options);
    } else {
        Options defaultOptions;
        return OpenSubdiv::Far::PatchTableFactory::Create(*refiner, defaultOptions);
    }
}

// PatchTable functions
void PatchTable_delete(PatchTable* table) {
    delete table;
}

int PatchTable_GetNumPatchArrays(const PatchTable* table) {
    return table->GetNumPatchArrays();
}

int PatchTable_GetNumPatches(const PatchTable* table) {
    return table->GetNumPatchesTotal();
}

int PatchTable_GetNumControlVertices(const PatchTable* table) {
    return table->GetNumControlVerticesTotal();
}

int PatchTable_GetMaxValence(const PatchTable* table) {
    return table->GetMaxValence();
}

// Local point functions
int PatchTable_GetNumLocalPoints(const PatchTable* table) {
    return table->GetNumLocalPoints();
}

const OpenSubdiv::Far::StencilTable* PatchTable_GetLocalPointStencilTable(const PatchTable* table) {
    return table->GetLocalPointStencilTable();
}

// Get patch array information
int PatchTable_GetNumPatches_PatchArray(const PatchTable* table, int arrayIndex) {
    if (arrayIndex < 0 || arrayIndex >= table->GetNumPatchArrays()) {
        return 0;
    }
    return table->GetNumPatches(arrayIndex);
}

// Get patch descriptor for a patch array
void PatchTable_GetPatchArrayDescriptor(const PatchTable* table, int arrayIndex, PatchDescriptor* desc) {
    if (arrayIndex < 0 || arrayIndex >= table->GetNumPatchArrays() || !desc) {
        return;
    }
    *desc = table->GetPatchArrayDescriptor(arrayIndex);
}

// Get patch control vertices
const int* PatchTable_GetPatchArrayVertices(const PatchTable* table, int arrayIndex) {
    if (arrayIndex < 0 || arrayIndex >= table->GetNumPatchArrays()) {
        return nullptr;
    }
    ConstIndexArray indices = table->GetPatchArrayVertices(arrayIndex);
    return &indices[0];
}

// Get patch param for a specific patch
void PatchTable_GetPatchParam(const PatchTable* table, int arrayIndex, int patchIndex, PatchParam* param) {
    if (!param) return;
    
    int handle = table->GetPatchArrayVertices(arrayIndex).size();
    if (patchIndex < 0 || patchIndex >= handle) {
        return;
    }
    
    *param = table->GetPatchParam(arrayIndex, patchIndex);
}

// Get all patch control vertex indices
const int* PatchTable_GetPatchControlVerticesTable(const PatchTable* table) {
    auto const& cvs = table->GetPatchControlVerticesTable();
    return cvs.empty() ? nullptr : &cvs[0];
}

// PatchTableFactory::Options functions
Options* PatchTableFactory_Options_new() {
    return new Options();
}

void PatchTableFactory_Options_delete(Options* options) {
    delete options;
}

void PatchTableFactory_Options_SetEndCapType(Options* options, int endCapType) {
    options->SetEndCapType(static_cast<Options::EndCapType>(endCapType));
}

int PatchTableFactory_Options_GetEndCapType(const Options* options) {
    return static_cast<int>(options->GetEndCapType());
}

void PatchTableFactory_Options_SetTriangleSubdivision(Options* /*options*/, int /*triangleSubdivision*/) {
    // Triangle subdivision is set through scheme type, not a separate option
    // This is a no-op for compatibility
}

void PatchTableFactory_Options_SetUseInfSharpPatch(Options* options, bool useInfSharpPatch) {
    options->useInfSharpPatch = useInfSharpPatch;
}

void PatchTableFactory_Options_SetNumLegacyGregoryPatches(Options* /*options*/, int /*numPatches*/) {
    // Legacy Gregory patches are not directly settable in newer versions
    // This is a no-op for compatibility
}

// PatchDescriptor functions
int PatchDescriptor_GetType(const PatchDescriptor* desc) {
    return static_cast<int>(desc->GetType());
}

int PatchDescriptor_GetNumControlVertices(const PatchDescriptor* desc) {
    return desc->GetNumControlVertices();
}

bool PatchDescriptor_IsRegular(const PatchDescriptor* desc) {
    // Check if it's a regular patch (bi-cubic B-spline)
    return desc->GetType() == PatchDescriptor::REGULAR;
}

// PatchParam functions
void PatchParam_GetUV(const PatchParam* param, float* u, float* v) {
    if (!u || !v) return;
    // Convert from log2 representation to normalized coordinates
    float depth = (float)(1 << param->GetDepth());
    *u = (float)param->GetU() / depth;
    *v = (float)param->GetV() / depth;
}

int PatchParam_GetDepth(const PatchParam* param) {
    return param->GetDepth();
}

bool PatchParam_IsRegular(const PatchParam* param) {
    return param->IsRegular();
}

int PatchParam_GetBoundary(const PatchParam* param) {
    return param->GetBoundary();
}

int PatchParam_GetTransition(const PatchParam* param) {
    return param->GetTransition();
}

} // extern "C"