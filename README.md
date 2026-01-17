# Andrea Mosaic

Turn your messy gallery into Andrea Mosaic.

here's an example i did with my own gallery. i used 471 images. however, more images you have, the better the result will be. at least 30–100 images for a rough mosaic. 200–1,000+ images for noticeably better matching.

```
cargo run -- generate --input input/scream.jpg --output output/mosaic.png --tiles catalog --tile-size 64
```

You can also use the CLI to generate the mosaic.

```
cargo run
```

![CLI](/images/cli.png)

> Navigate through the menu using the arrow keys and press enter to select.

Input 
![Input](/input/scream.jpg)

Output
![Andrea Mosaic](/output/output.png)

made by [Yaps GG](https://yaps.gg)