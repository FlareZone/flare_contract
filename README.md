# flare_contract
flare_app contract code

## flow chart

### version 0.0.1
https://www.oklink.com/cn/oktc-test/address/0x681F72164AC2079E233c1204260Ff10C4A5e4bcb

1. post creater to call publishPost that contract to create one gambling like | input _isBet true _betAmount 10000000000000000

2. gambling participant should call the participate | input payableAmount (ether) 0.01

3. everybody all can call | call endBet divide equally token value

### version 0.0.2
https://www.oklink.com/cn/oktc-test/address/0x880AdCf999F7f90ad4598704Bc487A76C62A5336

1. add user defined endtime, v0.0.1 is call by gambling starter time no defined

### version 0.0.3
https://www.oklink.com/cn/oktc-test/address/0x7ba0b3845b52fc8e588b64b6ab0178c079d3c4e3

1. add auto call endBet

### version 0.0.4
old: https://www.oklink.com/cn/oktc-test/address/0xce475a7b4a85b10530fc24ae13b1dd00657a98ae
new: https://www.oklink.com/cn/oktc-test/address/0xc0743e95f5cbe516bd7a90e8a4b8946185ccb750

1. Multiple bets are posted simultaneously
