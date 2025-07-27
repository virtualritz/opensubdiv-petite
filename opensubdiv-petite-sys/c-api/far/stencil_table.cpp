#include <opensubdiv/far/stencilTable.h>
#include <numeric>
#include <vector>

#include "../vtr/types.hpp"

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::Stencil Stencil;

typedef OpenSubdiv::Vtr::Index Index;

// Wrapper class that provides the required interface for UpdateValues
class FloatValue {
public:
    float value;
    
    FloatValue() : value(0.0f) {}
    
    void Clear() {
        value = 0.0f;
    }
    
    void AddWithWeight(const FloatValue& src, float weight) {
        value += src.value * weight;
    }
};

extern "C" {

    void StencilTable_destroy(StencilTable* st) {
        delete st;
    }

    /// Returns the number of stencils in the table
    int StencilTable_GetNumStencils(StencilTable* st) {
        return st->GetNumStencils();
    }

    /// \brief Returns the number of control vertices indexed in the table
    int StencilTable_GetNumControlVertices(StencilTable* st) {
        return st->GetNumControlVertices();
    }

    /// \brief Returns a Stencil at index i in the table
    Stencil StencilTable_GetStencil(StencilTable* st, Index i) {
        return st->GetStencil(i);
    }

    /// \brief Returns the number of control vertices of each stencil in the table
    IntVectorRef StencilTable_GetSizes(StencilTable* st) {
        auto& v = st->GetSizes();
        return IntVectorRef(v.data(), v.size());
    }

    /// \brief Returns the offset to a given stencil (factory may leave empty)
    IndexVectorRef StencilTable_GetOffsets(StencilTable* st) {
        auto& v = st->GetOffsets();
        return IndexVectorRef(v.data(), v.size());
    }

    /// \brief Returns the indices of the control vertices
    IndexVectorRef StencilTable_GetControlIndices(StencilTable* st) {
        auto& v = st->GetControlIndices();
        return IndexVectorRef(v.data(), v.size());
    }

    /// \brief Returns the stencil interpolation weights
    FloatVectorRef StencilTable_GetWeights(StencilTable* st) {
        auto& v = st->GetWeights();
        return FloatVectorRef(v.data(), v.size());
    }
    
    /// \brief Update values by applying the stencil table
    void StencilTable_UpdateValues(StencilTable* st, const float* src, float* dst, int start, int end) {
        int numControlVerts = st->GetNumControlVertices();
        int numStencils = st->GetNumStencils();
        
        // Create wrapper arrays
        std::vector<FloatValue> srcValues(numControlVerts);
        std::vector<FloatValue> dstValues(numStencils);
        
        // Copy input data to wrapper
        for (int i = 0; i < numControlVerts; ++i) {
            srcValues[i].value = src[i];
        }
        
        // Use the templated UpdateValues method with our wrapper type
        st->UpdateValues(srcValues.data(), dstValues.data(), start, end);
        
        // Copy results back
        int actualStart = (start < 0) ? 0 : start;
        int actualEnd = (end < 0 || end > numStencils) ? numStencils : end;
        
        for (int i = actualStart; i < actualEnd; ++i) {
            dst[i] = dstValues[i].value;
        }
    }
}