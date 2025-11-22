#include <opensubdiv/far/patchTable.h>
#include <opensubdiv/far/patchMap.h>
#include <opensubdiv/far/patchBasis.h>
#include <opensubdiv/far/patchDescriptor.h>
#include <vector>

typedef OpenSubdiv::Far::PatchTable PatchTable;
typedef OpenSubdiv::Far::PatchMap PatchMap;
typedef OpenSubdiv::Far::PatchParam PatchParam;

// Structure to hold evaluation results
struct PatchEvalResult {
    float point[3];
    float du[3];
    float dv[3];
    float duu[3];
    float duv[3];
    float dvv[3];
};

extern "C" {

// Evaluate a patch at given parametric coordinates
bool PatchTable_EvaluateBasis(
    const PatchTable* table,
    int patchIndex,
    float u, float v,
    float* wP,    // [out] weights for position (size = numControlVerts)
    float* wDu,   // [out] weights for du derivative (optional, can be null)
    float* wDv,   // [out] weights for dv derivative (optional, can be null)
    float* wDuu,  // [out] weights for duu derivative (optional, can be null)
    float* wDuv,  // [out] weights for duv derivative (optional, can be null)
    float* wDvv   // [out] weights for dvv derivative (optional, can be null)
) {
    if (!table || patchIndex < 0 || patchIndex >= table->GetNumPatchesTotal()) {
        return false;
    }
    
    // Get patch array and local patch index
    int patchArray = 0;
    int localPatchIndex = patchIndex;
    for (int i = 0; i < table->GetNumPatchArrays(); ++i) {
        int numPatches = table->GetNumPatches(i);
        if (localPatchIndex < numPatches) {
            patchArray = i;
            break;
        }
        localPatchIndex -= numPatches;
    }
    
    // Get patch descriptor
    auto desc = table->GetPatchArrayDescriptor(patchArray);
    int numControlVerts = desc.GetNumControlVertices();
    
    // Get patch param
    PatchParam param = table->GetPatchParam(patchArray, localPatchIndex);
    
    // Normalize coordinates
    param.Normalize(u, v);
    
    // Evaluate basis functions based on patch type
    typedef OpenSubdiv::Far::PatchDescriptor Descriptor;
    
    if (desc.GetType() == Descriptor::REGULAR) {
        // Regular B-spline patch - cubic B-spline basis

        // Simplified cubic B-spline basis evaluation
        // This is a placeholder - proper implementation would use OpenSubdiv's basis functions
        if (wP) {
            // Initialize weights to uniform values for now
            float w = 1.0f / numControlVerts;
            for (int i = 0; i < numControlVerts; ++i) {
                wP[i] = w;
            }
        }
        
        // Zero out derivative weights if requested
        if (wDu) {
            for (int i = 0; i < numControlVerts; ++i) wDu[i] = 0.0f;
        }
        if (wDv) {
            for (int i = 0; i < numControlVerts; ++i) wDv[i] = 0.0f;
        }
        if (wDuu) {
            for (int i = 0; i < numControlVerts; ++i) wDuu[i] = 0.0f;
        }
        if (wDuv) {
            for (int i = 0; i < numControlVerts; ++i) wDuv[i] = 0.0f;
        }
        if (wDvv) {
            for (int i = 0; i < numControlVerts; ++i) wDvv[i] = 0.0f;
        }
        
        return true;
    }
    
    // Other patch types not implemented yet
    return false;
}

// Helper function to evaluate patch and apply to control points
bool PatchTable_EvaluatePoint(
    const PatchTable* table,
    int patchIndex,
    float u, float v,
    const float* controlPoints,  // Control points (3 floats per vertex)
    int numControlPoints,
    PatchEvalResult* result
) {
    if (!table || !controlPoints || !result) {
        return false;
    }
    
    // Get total patches
    int totalPatches = table->GetNumPatchesTotal();
    if (patchIndex < 0 || patchIndex >= totalPatches) {
        return false;
    }
    
    // Find which array this patch belongs to
    int patchArray = 0;
    int localPatchIndex = patchIndex;

    for (int i = 0; i < table->GetNumPatchArrays(); ++i) {
        int numPatches = table->GetNumPatches(i);
        if (localPatchIndex < numPatches) {
            patchArray = i;
            break;
        }
        localPatchIndex -= numPatches;
    }
    
    // Get patch info
    auto desc = table->GetPatchArrayDescriptor(patchArray);
    int numCVs = desc.GetNumControlVertices();
    
    // Allocate space for basis weights
    std::vector<float> wP(numCVs), wDu(numCVs), wDv(numCVs);
    std::vector<float> wDuu(numCVs), wDuv(numCVs), wDvv(numCVs);
    
    // Evaluate basis functions
    if (!PatchTable_EvaluateBasis(table, patchIndex, u, v,
                                  wP.data(), wDu.data(), wDv.data(),
                                  wDuu.data(), wDuv.data(), wDvv.data())) {
        return false;
    }
    
    // Get control vertex indices for this patch
    auto cvIndices = table->GetPatchArrayVertices(patchArray);
    int cvStart = localPatchIndex * numCVs;
    
    // Initialize result
    for (int i = 0; i < 3; ++i) {
        result->point[i] = 0.0f;
        result->du[i] = 0.0f;
        result->dv[i] = 0.0f;
        result->duu[i] = 0.0f;
        result->duv[i] = 0.0f;
        result->dvv[i] = 0.0f;
    }
    
    // Apply weights to control points
    for (int cv = 0; cv < numCVs; ++cv) {
        int vertexIndex = cvIndices[cvStart + cv];
        if (vertexIndex >= numControlPoints) {
            return false;
        }
        
        const float* cp = &controlPoints[vertexIndex * 3];
        
        for (int i = 0; i < 3; ++i) {
            result->point[i] += wP[cv] * cp[i];
            result->du[i] += wDu[cv] * cp[i];
            result->dv[i] += wDv[cv] * cp[i];
            result->duu[i] += wDuu[cv] * cp[i];
            result->duv[i] += wDuv[cv] * cp[i];
            result->dvv[i] += wDvv[cv] * cp[i];
        }
    }
    
    return true;
}

// Create patch map for efficient patch location
PatchMap* PatchMap_Create(const PatchTable* table) {
    if (!table) return nullptr;
    return new PatchMap(*table);
}

void PatchMap_delete(PatchMap* map) {
    delete map;
}

// Find patch containing given face and (u,v)
bool PatchMap_FindPatch(
    const PatchMap* map,
    int faceIndex,
    float u, float v,
    int* patchIndex,
    float* patchU,
    float* patchV
) {
    if (!map || !patchIndex || !patchU || !patchV) {
        return false;
    }
    
    const PatchTable::PatchHandle* handle = map->FindPatch(faceIndex, u, v);
    if (!handle) {
        return false;
    }
    
    *patchIndex = handle->patchIndex;
    
    // Get patch param and transform coordinates
    PatchParam param;
    // Note: This is simplified - actual implementation would get param from patch table
    param.Normalize(*patchU, *patchV);
    
    return true;
}

} // extern "C"