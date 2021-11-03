
# Sine Chat

一个小型即时通信服务（练手项目）。

![demo](https://github.com/TangentW/sine_chat/blob/3f851cde01131159c761c1ab9e21b83274fefc03/imgs/demo.gif)

---

## 简介

 `Sine Chat` 是一个我用于练手 Rust 的项目，网络侧基于 TCP 进行数据传输；在工程上基于 Rust 的 Future 使用了异步的编码方式（协程），并以 `tokio` 作为异步的 runtime。

Demo 展示：

## 帧

因为 TCP 基于字节流，所以应用层上需要拟定数据帧（frame）进行数据包划分。`Sine Chat` 的数据帧设计如下：

![frame](https://github.com/TangentW/sine_chat/blob/3f851cde01131159c761c1ab9e21b83274fefc03/imgs/frame.png)

帧主要分成两大部分：`帧头（header）`和`数据载荷（payload）`。

帧头总占 40 位（5 字节），再划分成两部分：
* `帧类型`：占 8 位，表示数据载荷的类型
* `数据载荷长度`：用于框定帧的数据载荷边界

帧类型与对应数据结构的映射表如下：

| Type Code | From Client | From Server |
| :---: | :----: | :---: |
| 0x00 | Handshake | HandshakeReply |
| 0x01 | ClientMessage | ServerMessage |
| 0x02 | N/A | MessageReply |
| 0xFF | Ping | Pong |
| 0x03 ~ 0xFE | [Reserved] | [Reserved] |

## 通信流

![flow](https://github.com/TangentW/sine_chat/blob/3f851cde01131159c761c1ab9e21b83274fefc03/imgs/flow.png)

### 握手阶段

客户端在与服务端 TCP 连接后，需要进行 `Sine Chat` 侧的握手（handshake），客户端此时需向服务端传递鉴权信息（token），以向服务端表明身份。服务端收到握手信息后进行鉴权处理，继而向客户端发送握手响应。若客户端在与服务端建立 TCP 连接后长时间不进行握手，服务端则会主动关闭 TCP 连接。

如服务端校验握手信息成功，客户端则进入在线状态。

因 `Sine Chat` 尚未接入数据库，且作为练手项目，为了简单，所以这里握手的鉴权处理则直接把客户端传入的 token 作为客户端用户名，以用于后续的客户端间消息传递。

### 在线阶段

客户端握手成功后，可以发送消息，消息需要拟定接收方，为了简单，这里直接指定为接收方客户端的用户名。

发送方客户端发送的消息传至服务端后，服务端会继而给客户端发送消息回应。

服务端在收到消息后会对消息进行解码校验，然后寻找消息所指定的接收方。如果消息解码校验有误或找不到接收方，服务端则会把错误信息置入消息回应中返回给发送方客户端。

最后，消息发送方和接收方客户端都能收到消息的内容。

