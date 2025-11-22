// Minimal wrapper for patch table functionality
extern "C" {
    // Forward declarations for opaque types
    typedef struct OpenSubdiv__v3_7_0__Far__PatchTable PatchTable;
    typedef struct OpenSubdiv__v3_7_0__Far__TopologyRefiner TopologyRefiner;
    typedef struct OpenSubdiv__v3_7_0__Far__PatchTableFactory__Options PatchTableFactoryOptions;
    typedef struct OpenSubdiv__v3_7_0__Far__PatchDescriptor PatchDescriptor;
    typedef struct OpenSubdiv__v3_7_0__Far__PatchParam PatchParam;

    // PatchTableFactory functions
    PatchTable* PatchTableFactory_Create(TopologyRefiner* refiner, PatchTableFactoryOptions* options);

    // PatchTable functions
    void PatchTable_delete(PatchTable* table);
    int PatchTable_GetNumPatchArrays(const PatchTable* table);
    int PatchTable_GetNumPatches(const PatchTable* table);
    int PatchTable_GetNumControlVertices(const PatchTable* table);
    int PatchTable_GetMaxValence(const PatchTable* table);
    int PatchTable_GetNumPatches_PatchArray(const PatchTable* table, int array_index);
    void PatchTable_GetPatchArrayDescriptor(const PatchTable* table, int array_index, PatchDescriptor* desc);
    const int* PatchTable_GetPatchArrayVertices(const PatchTable* table, int array_index);
    void PatchTable_GetPatchParam(const PatchTable* table, int array_index, int patch_index, PatchParam* param);
    const int* PatchTable_GetPatchControlVerticesTable(const PatchTable* table);

    // PatchTableFactory::Options functions
    PatchTableFactoryOptions* PatchTableFactory_Options_new();
    void PatchTableFactory_Options_delete(PatchTableFactoryOptions* options);
    void PatchTableFactory_Options_SetEndCapType(PatchTableFactoryOptions* options, int end_cap_type);
    int PatchTableFactory_Options_GetEndCapType(const PatchTableFactoryOptions* options);
    void PatchTableFactory_Options_SetTriangleSubdivision(PatchTableFactoryOptions* options, int triangle_subdivision);
    void PatchTableFactory_Options_SetUseInfSharpPatch(PatchTableFactoryOptions* options, bool use_inf_sharp_patch);
    void PatchTableFactory_Options_SetNumLegacyGregoryPatches(PatchTableFactoryOptions* options, int num_patches);

    // PatchDescriptor functions
    int PatchDescriptor_GetType(const PatchDescriptor* desc);
    int PatchDescriptor_GetNumControlVertices(const PatchDescriptor* desc);
    bool PatchDescriptor_IsRegular(const PatchDescriptor* desc);

    // PatchParam functions
    void PatchParam_GetUV(const PatchParam* param, float* u, float* v);
    int PatchParam_GetDepth(const PatchParam* param);
    bool PatchParam_IsRegular(const PatchParam* param);
    int PatchParam_GetBoundary(const PatchParam* param);
    int PatchParam_GetTransition(const PatchParam* param);
}