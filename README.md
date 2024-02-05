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

```shell
$ jq .[].to_url data/links.json | sort | uniq --count --repeated | sort | tail
   37 "https://itunes.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528"
   76 "https://static.thetype.cloud/typechat/assets/typechat-weapp.jpg"
   83 "https://www.thetype.com/typechat/feed/"
   89 "https://podcasts.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528"
   95 "https://itunes.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528"
  104 "https://www.thetype.com/members/"
  138 "https://www.thetype.com/feed/typechat/"
  171 "/cdn-cgi/l/email-protection"
  184 "http://music.163.com/#/djradio?id=346541057"
  207 "http://www.lizhi.fm/1852153/"
```
