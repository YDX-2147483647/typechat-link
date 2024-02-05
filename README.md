# 《字谈字畅》链接

```shell
$ cargo run
Loading episodes from data/episodes.json…
Loading links from data/links.json…
Data:
  221 episodes.
  221 links. (808 point to thetype.com, 104 point to typechat)
Saving to out/typechat.dot…

$ dot out/typechat.dot -Tsvg -o out/typechat.svg
```

初次运行时`data/*.json`不存在，会自动从网上获取。
