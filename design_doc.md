## Message Queue
當系統跟系統間的溝通不是即時時，message queue是一個非常理想用來「暫存」請求的一個媒介。發出需求的系統首先向message queue傳送訊息，接受訊息的系統可以在準備好的情況下從message queue「拉」出來自其他系統的請求。

更甚者，不同的系統可以向同一個message重複讀取相同的訊息來做不同的事情。舉例，當使用者向uber eats發出送餐請求時，負責處理訂單的系統可以聯絡商家和騎手，負責大數據處理的系統也可以向同一個message queue取得這個請求來統計例如不同商家的受歡迎情形。

這個message queue將會透過三個部分來實作：segment、topic以及consumer group。
consumer group負責管理不同的consumer group，不同的consumer會記錄不同的進度(offset)；
topic負責管理該topic下面的資訊諸如segment的base directory以及其下的所有segment；
segment負責實際的訊息儲存方式。

另外還會有message的物件來儲存message本身。

以下將按照segment -> topic -> consumer group的順序來實作這個message queue。

### Segment

