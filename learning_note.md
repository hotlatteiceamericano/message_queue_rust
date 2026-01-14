1. 要寫入message的長度時，也要用二進位的形式寫入。選用big endian的原因是符合人看的方式，最大的位元擺在前面。
2. 因為一個topic下面可能有很多segment，透過紀錄base_offset來知道一個global offset會落在哪一個segment
3. 使用rstest的fixture在統一的地方宣稱test dependency objects
4. topic用segment的base offset來找下一個write，和哪裡read messages
5. topic需要管理global write offset；consumer group管理global read offset per consumer group
6. 使用range在BTreeMap裡面尋找時，回傳的是tupple而不是entry。
    - 我覺得一個比較關鍵的原因是因為Entry是owned value，如果因為我的遍歷而擁有了這個BTreeMap裡面的所有值感覺也不合理
7. 因為回傳的是tuple的關係，接續用map做處理時也記得互動的是tuple type
8. topic找target_segment那邊學到善用match來簡化程式碼
