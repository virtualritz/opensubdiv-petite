// Minimal C API shim for Bfr::RefinerSurfaceFactory and Surface (float).
#include <opensubdiv/bfr/refinerSurfaceFactory.h>
#include <opensubdiv/bfr/surface.h>
#include <opensubdiv/far/topologyRefiner.h>
#include <vector>

using namespace OpenSubdiv;          // NOLINT
using namespace OPENSUBDIV_VERSION;  // NOLINT

extern "C"
{

    // Opaque wrappers for Rust.
    struct Bfr_SurfaceFactory_f
    {
        Bfr::RefinerSurfaceFactory<> *ptr;
    };

    struct Bfr_Surface_f
    {
        Bfr::Surface<float> surface;
    };

    Bfr_SurfaceFactory_f *Bfr_SurfaceFactory_Create(
        Far::TopologyRefiner *refiner, int approx_level_smooth, int approx_level_sharp)
    {
        if (!refiner) {
            return nullptr;
        }

        Bfr::SurfaceFactory::Options opts;
        opts.SetApproxLevelSmooth(approx_level_smooth);
        opts.SetApproxLevelSharp(approx_level_sharp);

        auto *wrapper = new Bfr_SurfaceFactory_f();
        wrapper->ptr = new Bfr::RefinerSurfaceFactory<>(*refiner, opts);
        return wrapper;
    }

    void Bfr_SurfaceFactory_Destroy(Bfr_SurfaceFactory_f *factory)
    {
        if (!factory)
            return;
        delete factory->ptr;
        delete factory;
    }

    Bfr_Surface_f *Bfr_Surface_Create()
    {
        return new Bfr_Surface_f();
    }

    void Bfr_Surface_Destroy(Bfr_Surface_f *surface)
    {
        delete surface;
    }

    bool Bfr_SurfaceFactory_InitVertexSurface(
        const Bfr_SurfaceFactory_f *factory, int face_index, Bfr_Surface_f *surface)
    {
        if (!factory || !factory->ptr || !surface) {
            return false;
        }

        surface->surface.Clear();
        return factory->ptr->InitVertexSurface(face_index, &surface->surface);
    }

    bool Bfr_Surface_IsValid(const Bfr_Surface_f *surface)
    {
        return surface && surface->surface.IsValid();
    }

    bool Bfr_Surface_IsRegular(const Bfr_Surface_f *surface)
    {
        return surface && surface->surface.IsRegular();
    }

    int Bfr_Surface_GetNumControlPoints(const Bfr_Surface_f *surface)
    {
        if (!surface)
            return 0;
        return surface->surface.GetNumControlPoints();
    }

    int Bfr_Surface_GetControlPointIndices(
        const Bfr_Surface_f *surface, int *out_indices, int max_count)
    {
        if (!surface || !out_indices || max_count <= 0) {
            return 0;
        }
        return surface->surface.GetControlPointIndices(out_indices);
    }

    // Evaluate position at (u,v) using mesh points with stride 3 floats.
    bool Bfr_Surface_EvaluatePosition(
        const Bfr_Surface_f *surface,
        float u,
        float v,
        const float *mesh_points,
        int mesh_stride,
        float *out_p3)
    {
        if (!surface || !mesh_points || !out_p3) {
            return false;
        }
        if (!surface->surface.IsValid()) {
            return false;
        }

        Bfr::Surface<float>::PointDescriptor mesh_desc(3, mesh_stride);
        Bfr::Surface<float>::PointDescriptor patch_desc(3);

        const int num_patch_pts = surface->surface.GetNumPatchPoints();
        std::vector<float> patch_points(static_cast<size_t>(num_patch_pts) * 3);
        surface->surface.PreparePatchPoints(
            mesh_points, mesh_desc, patch_points.data(), patch_desc);

        float uv[2] = {u, v};
        surface->surface.Evaluate(uv, patch_points.data(), patch_desc, out_p3);
        return true;
    }

    // Expose patch point count for buffer sizing.
    int Bfr_Surface_GetNumPatchPoints(const Bfr_Surface_f *surface)
    {
        if (!surface)
            return 0;
        return surface->surface.GetNumPatchPoints();
    }

    bool Bfr_Surface_GatherPatchPoints(
        const Bfr_Surface_f *surface,
        const float *mesh_points,
        int mesh_stride,
        float *out_patch_points,
        int max_points)
    {
        if (!surface || !mesh_points || !out_patch_points) {
            return false;
        }
        if (!surface->surface.IsValid()) {
            return false;
        }

        const int num_patch_pts = surface->surface.GetNumPatchPoints();
        if (num_patch_pts > max_points) {
            return false;
        }

        Bfr::Surface<float>::PointDescriptor mesh_desc(3, mesh_stride);
        Bfr::Surface<float>::PointDescriptor patch_desc(3);

        std::vector<float> patch_points(static_cast<size_t>(num_patch_pts) * 3);
        surface->surface.PreparePatchPoints(
            mesh_points, mesh_desc, patch_points.data(), patch_desc);

        std::memcpy(
            out_patch_points, patch_points.data(), patch_points.size() * sizeof(float));
        return true;
    }
}
