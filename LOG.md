Every wasted second due to software bugs or crap tooling is logged here.


* 17/05/2023 - 1h - tried to recompile kyute
   * error finding openimageio in vcpkg; reinstall openimageio via vcpkg
   * error in build script for openimageio-sys bindings: open openimageio-sys project, rebuild, but the build script doesn't show any errors
   * bug in cc-rs if linker output contains non-UTF8 characters
   * open a bug on github
   * total time wasted: ~1h

* 20/05/2023 - 15min - linker error with skia-safe
   * `error LNK2019: unresolved external symbol __std_max_element_4` 
   * github issue suggests to update MSVC compiler, works
  
* 21/05/2023 - 4h - coding error but with unclear error message
   * made a mistake with a windows API: created a compositor, a CompositionGraphicsDevice, but then replaced the original compositor with another
   * got into a rabbit hole trying different combinations of APIs

* 27/10/2023 - 4h+ - egui-winit not up-to-date / WGPU not working with D3D12 or Vulkan
  * Cargo patch doesn't work since the versions (0.28/0.29) are different
  * fork egui-winit / actually fork egui entirely since they are in the same repo
  * upgrade winit
  * bunch of types have changed: e.g. a completely gratuitous breakage on the `Modifiers` type that removed the `ctrl`, `alt` methods for no good reason
  * finally fix every compilation issue
  * D3D12 WGPU backend triggers a "DeviceLost" issue in kyute compositor code
  * Vulkan WGPU backend fails in Surface::configure, possibly with an ERROR_SURFACE_LOST_KHR somewhere
  * everything graphics-related is still fucked up


