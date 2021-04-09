#include <opensubdiv/far/stencilTable.h>

#include "../vtr/types.hpp"

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::Stencil Stencil;

typedef OpenSubdiv::Vtr::Index Index;

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
}