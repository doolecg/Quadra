# Quadra

A free, open-source 3D modelling, animation and rendering app for Windows that works like 3ds Max 2026: same layout, same shortcuts, same workflows. Written in Rust, rendering with Cycles.

Early days: milestone 1 (shell and viewport) is in progress. The main window opens, but it cannot create or edit anything yet.

## Get started

Needs Rust stable (1.85 or newer) on Windows.

```
git clone https://github.com/doolecg/Quadra
cd Quadra
cargo run -p quadra_app
```

That opens the main window. The plan and progress live in `docs/bible.html`.

## Credits

Quadra copies the way 3ds Max works so Max users feel at home. It uses no Autodesk code, text, icons or artwork, and reads no `.max` files. Rendering will use [Cycles](https://www.cycles-renderer.org/) (Apache-2.0).

## Licence

Apache-2.0, see [LICENSE](LICENSE).

## Disclaimer

This software is provided "as is", without warranty of any kind, as set out in the [LICENSE](LICENSE). To the extent permitted by law, the authors are not liable for any loss or damage from using it, including lost data, lost work or damage to files or projects. Nothing here limits liability that cannot be limited by law, such as for death or personal injury caused by negligence.

Back up your work before using it. You use it at your own risk.

Not affiliated with or endorsed by Autodesk. 3ds Max is a trademark of Autodesk, Inc.
