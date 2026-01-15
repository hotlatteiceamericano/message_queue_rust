# Message Queue
當系統跟系統間的溝通不是即時時，message queue是一個非常理想用來「暫存」請求的一個媒介。發出需求的系統首先向message queue傳送訊息，接受訊息的系統可以在準備好的情況下從message queue「拉」出來自其他系統的請求。

更甚者，不同的系統可以向同一個message重複讀取相同的訊息來做不同的事情。舉例，當使用者向uber eats發出送餐請求時，負責處理訂單的系統可以聯絡商家和騎手，負責大數據處理的系統也可以向同一個message queue取得這個請求來統計例如不同商家的受歡迎情形。

Message queue常會需要支援極大量的讀寫需求，partition是支持大量讀寫的最佳辦法。為了學習和簡化的目的，這個project不會實作partition的功能。

## Segment
Segment負責把message以二進位的形式寫入在local file system，以及將message從二進位的形式讀取、並以message物件的方式回傳。
Segment需要記錄`base_offset: u64`, `write_position: u64`和`file` fields。base_offset用來記錄這個segment在一個topic下可能是第幾個segment，write_position用來記錄這個segment本身的offset是多少。

### Segment::write(message: Message) -> Result<u64>
write method接受Message參數，使用serde library將message的content轉化成二進制的方式並儲存到該segment對應的實體檔案。
write method會回傳Result<u64>，Ok<u64>代表寫入成功、並且告訴topic這個segment最新的offset到多少；Err則代表寫入失敗。

### Segment::read(offset: u64) -> Result<Message>
read method會接受offset做參數，並回傳Result<Message>。Err需要區分是提供的offset找不到相應的message，或是其他讀取上的錯誤。

## Topic
Topic管理不同類型的訊息。Topic需要追蹤該topic下面有幾個segment、active segment，和目前的write offset。同樣需要提供write和read兩個methods。

### Topic::write(message: Message) -> Result<_>
接受Message當做參數。根據目前的write offset找到active segment，並呼叫該segment的write method，會根據Segment::write()來更新目前的write offset。

### Topic::read(offset: u64) -> Result<Message>
接受offset當做參數，並回傳Result<Message>。Err需要區分是提供的offset找不到相應的message，或是其他讀取上的錯誤。

## Consumer Group
負責管理每個consumer group的讀什麼topic到多少offset，將以HashMap<Topic, u64>的方式為每個consumer group儲存哪一個topic讀到哪一個offset。
需要注記這裡記錄的offset是「read offset」，不同的Topic所記錄的offset是「write offset」。
consumer group同樣需要提供write和read methods。

### ConsumerGroup::write(topic_name: String, message: Message) -> Result<_>
接受topic name和Message當做參數，首先透過Topic::from檢查該Topic是否存在，因此，會需要一個topic.save()來儲存既有的topic。topic會自身的field以json的格式儲存在每個topic的目錄下面，以利Topic::from讀取。

接著呼叫Topic::write將message寫入到最新的write offset。最後回傳Result<_>來表達寫入的成功與否。

### ConsumerGroup::read(topic: Topic) -> Result<Message>
接受Topic當作參數，並根據HashMap的read offset來讀取最新的message。會回傳Result<Message>給client。

備註，透過決定讓ConsumerGroup::read()和ConsumerGroup::write()接受Message當做參數，代表的是我將提供Message型態當作client也可以用的public型態。

## Message
目前先用content的名字和String的型態來記錄這個message，將適時擴展這個型態。
