#include <opensubdiv/far/primvarRefiner.h>
#include <opensubdiv/sdc/options.h>
#include <opensubdiv/sdc/types.h>

#include <stdio.h>

typedef OpenSubdiv::Far::PrimvarRefiner PrimvarRefiner;
typedef OpenSubdiv::Far::TopologyRefiner TopologyRefiner;

template <int N> struct Primvar {
    float v[N];

    void Clear() {
        for (int i = 0; i < N; ++i) {
            v[i] = 0.0f;
        }
    }

    void AddWithWeight(const Primvar& p, float weight) {
        for (int i = 0; i < N; ++i) {
            v[i] += p.v[i] * weight;
        }
    }
};

extern "C" {
    PrimvarRefiner* PrimvarRefiner_create(TopologyRefiner* tr) {
        return new PrimvarRefiner(*tr);
    }

    // FIXME: { if the TopologyRefiner is never fed to a PrimvarRefiner it will
    // leak?
    void PrimvarRefiner_destroy(PrimvarRefiner* pr) { delete pr; }

    const TopologyRefiner* PrimvarRefiner_GetTopologyRefiner(PrimvarRefiner* pr) {
        return &pr->GetTopologyRefiner();
    }

    void PrimvarRefiner_Interpolate(
        PrimvarRefiner* pr,
        int num_elements,
        int level,
        float* src,
        float* dst
    ) {
        Primvar<1>* dst1;
        Primvar<2>* dst2;
        Primvar<3>* dst3;
        Primvar<4>* dst4;

        switch (num_elements) {
            case 1: {
                dst1 = (Primvar<1>*)dst;
                pr->Interpolate(level, (Primvar<1>*)src, dst1);
                break;
            }
            case 2: {
                dst2 = (Primvar<2>*)dst;
                pr->Interpolate(level, (Primvar<2>*)src, dst2);
                break;
            }
            case 3: {
                dst3 = (Primvar<3>*)dst;
                pr->Interpolate(level, (Primvar<3>*)src, dst3);
                break;
            }
            case 4: {
                dst4 = (Primvar<4>*)dst;
                pr->Interpolate(level, (Primvar<4>*)src, dst4);
                break;
            }
            default: {
                printf("Invalid num elements for Interpolate: { %d\n", num_elements);
                std::terminate();
            }
        }
    }

    void PrimvarRefiner_InterpolateVarying(
        PrimvarRefiner* pr,
        int num_elements,
        int level,
        float* src,
        float* dst
    ) {
        Primvar<1>* dst1;
        Primvar<2>* dst2;
        Primvar<3>* dst3;
        Primvar<4>* dst4;

        switch (num_elements) {
            case 1: {
                dst1 = (Primvar<1>*)dst;
                pr->InterpolateVarying(level, (Primvar<1>*)src, dst1);
                break;
            }
            case 2: {
                dst2 = (Primvar<2>*)dst;
                pr->InterpolateVarying(level, (Primvar<2>*)src, dst2);
                break;
            }
            case 3: {
                dst3 = (Primvar<3>*)dst;
                pr->InterpolateVarying(level, (Primvar<3>*)src, dst3);
                break;
            }
            case 4: {
                dst4 = (Primvar<4>*)dst;
                pr->InterpolateVarying(level, (Primvar<4>*)src, dst4);
                break;
            }
            default: {
                printf("Invalid num elements for InterpolateVarying: { %d\n", num_elements);
                std::terminate();
            }
        }
    }

    void PrimvarRefiner_InterpolateFaceUniform(
        PrimvarRefiner* pr,
        int num_elements,
        int level,
        float* src,
        float* dst
    ) {
        Primvar<1>* dst1;
        Primvar<2>* dst2;
        Primvar<3>* dst3;
        Primvar<4>* dst4;

        switch (num_elements) {
            case 1: {
                dst1 = (Primvar<1>*)dst;
                pr->InterpolateFaceUniform(level, (Primvar<1>*)src, dst1);
                break;
            }
            case 2: {
                dst2 = (Primvar<2>*)dst;
                pr->InterpolateFaceUniform(level, (Primvar<2>*)src, dst2);
                break;
            }
            case 3: {
                dst3 = (Primvar<3>*)dst;
                pr->InterpolateFaceUniform(level, (Primvar<3>*)src, dst3);
                break;
            }
            case 4: {
                dst4 = (Primvar<4>*)dst;
                pr->InterpolateFaceUniform(level, (Primvar<4>*)src, dst4);
                break;
            }
            default: {
                printf("Invalid num elements for InterpolateFaceUniform: { %d\n", num_elements);
                std::terminate();
            }
        }
    }

    void PrimvarRefiner_InterpolateFaceVarying(
        PrimvarRefiner* pr,
        int num_elements,
        int level,
        float* src,
        float* dst
    ) {
        Primvar<1>* dst1;
        Primvar<2>* dst2;
        Primvar<3>* dst3;
        Primvar<4>* dst4;

        switch (num_elements) {
            case 1: {
                dst1 = (Primvar<1>*)dst;
                pr->InterpolateFaceVarying(level, (Primvar<1>*)src, dst1);
                break;
            }
            case 2: {
                dst2 = (Primvar<2>*)dst;
                pr->InterpolateFaceVarying(level, (Primvar<2>*)src, dst2);
                break;
            }
            case 3: {
                dst3 = (Primvar<3>*)dst;
                pr->InterpolateFaceVarying(level, (Primvar<3>*)src, dst3);
                break;
            }
            case 4: {
                dst4 = (Primvar<4>*)dst;
                pr->InterpolateFaceVarying(level, (Primvar<4>*)src, dst4);
                break;
            }
            default: {
                printf("Invalid num elements for InterpolateFaceVarying: { %d\n", num_elements);
                std::terminate();
            }
        }
    }
}