# 《字谈字畅》链接

[《字谈字畅》（_TypeChat_）][typechat]是全球首家用华语制作的字体排印主题播客节目。此项目从其网站爬取各期的参考链接，并整理出一些信息。

- [`data/`][release-data]——原始数据
  - `episodes.json`——每一期的`name`、`url`和参考链接
  - `short_urls.json`——短链接缓存
- `out/`——输出文件
  - [`typechat.dot`][release-dot]——各期之间的链接关系图（可用 [Graphviz][graphviz] 生成[`typechat.svg`][release-svg]）
  - [`external-links.md`][release-md]——频繁引用的外部链接排名

```shell
$ cargo run
Loading episodes from data/episodes.json…
Loading short URL cache from data/short_urls.json…
✅ Found 247 episodes.

✅ Found 7473 links.

Saving to out/external-links.md…

Saving to out/typechat.dot…

$ dot out/typechat.dot -Tsvg -o out/typechat.svg
```

初次运行时`data/*.json`不存在，会自动从网上获取。

[typechat]: https://www.thetype.com/typechat/
[graphviz]: https://graphviz.org/

[release-data]: https://github.com/YDX-2147483647/typechat-link/releases/latest/download/data.7z
[release-dot]: https://github.com/YDX-2147483647/typechat-link/releases/latest/download/typechat.dot
[release-svg]: https://github.com/YDX-2147483647/typechat-link/releases/latest/download/typechat.svg
[release-md]: https://github.com/YDX-2147483647/typechat-link/releases/latest/download/external-links.md

## 冷知识

- Apple 播客链接有三种写法：

  - [itunes.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528](https://itunes.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528)
  - [itunes.apple.com/cn/podcast/字谈字畅/id1041704528](https://itunes.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528)
  - [podcasts.apple.com/cn/podcast/字谈字畅/id1041704528](https://podcasts.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528)

  其实还有第四种写法：

  - [podcasts.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528](https://podcasts.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528)

  ```mermaid
  flowchart LR
    itunes/pin-yi[itunes.apple.com<br>拼音]
    -->|301<br>Moved Permanently| podcasts/pin-yi([podcasts.apple.com<br>拼音])
    -->|301<br>Moved Permanently| podcasts/汉字[podcasts.apple.com<br>汉字]
    itunes/汉字[itunes.apple.com<br>汉字]
    -->|301<br>Moved Permanently| podcasts/汉字
  ```

- [第1期](https://www.thetype.com/typechat/ep-001/)结尾链接间的分割线也是`<a>`。

  ```html
  <p class="noindent">
    <a href="https://www.thetype.com/feed/typechat/">订阅地址</a>
    <a>｜</a>
    <a href="https://itunes.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528">iTunes</a>
    <a>｜</a>
    <a href="https://static.thetype.cloud/typechat/typechat001.mp3">下载音频</a>
    <a></a>
  </p>
  ```

- [第39期](https://www.thetype.com/typechat/ep-039/)“写真歴史博物館”的地址也是`写真歴史博物館`，于是被浏览器当作[`./写真歴史博物館`](https://www.thetype.com/typechat/ep-039/%E5%86%99%E7%9C%9F%E6%AD%B4%E5%8F%B2%E5%8D%9A%E7%89%A9%E9%A4%A8)：

  > **Not Found**
  >
  > Apologies, but we were unable to find what you were looking for. Perhaps searching will help.


- 可能因为部分平台字数限制太严，[第35期](https://www.thetype.com/typechat/ep-035/)采用了`t.cn`短链接服务，例如 [t.cn/zHVwH1H](https://t.cn/zHVwH1H)：

  > **将要访问**
  >
  > `https://en.wikipedia.org/wiki/Emoji`
  >
  > 此页面未在微博完成域名备案，可能存在内容风险。
  >
  > 如要继续访问，请注意你的隐私及财产安全。

  不过其它期似乎都没用。
