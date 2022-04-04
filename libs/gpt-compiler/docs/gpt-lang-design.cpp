
// Hierarchia, mindkét feature-ben kell a VIP
// Feature 1: Price calculation
[
  var VIP: bool 
  var price: num
  var second_hand_price: num

  if(VIP = true && price < 50 && price != 0) {
    if(price < 20 && second_hand_price > 60)
    if(price != 50)
  }
  else if(second_hand_price > 60)

  // This is "context free", it doesn't depend on higher hierarchies
  if(price > 30 && second_hand_price > 60)

  if(VIP = true) {
    if(second_hand_price = 2)
    if(second_hand_price = 3)
  }

  if(second_hand_price >= 50) {
    if(price < 5)
  }
  else if(10 < second_hand_price)

  if(price in [0,10] && price not in (9,100])

  // or
  if(VIP = true || price < 10) {
    if(second_hand_price = 2)
    if(second_hand_price = 3)
  }
  // becomes:
  if(VIP = true) {
    if(second_hand_price = 2)
    if(second_hand_price = 3)
  }
  if(price < 10) {
    if(second_hand_price = 2)
    if(second_hand_price = 3)
  }


  if(price > 10) {
    if(price < 100) {
      if(price in [20,30])
    }
  }

  if(price in (-Inf,0) || price in (0,10]) // price < 10 && price != 0

  // Feature 1.1: Shipping
  [
    var shipping: bool
    if(VIP = true && shipping = true)
  ]
]
// Feature 2: Credit card charging
[
  var has_credit_card: bool
  var credit_card_balance: int
  if(has_credit_card = true && credit_card_balance >= 10)
]


var a: int
var b: int
var c: int
var d: int

[(a)
  if(a > 10)
]
[(b)
  if (b < 20)
]
[(a,b)
  if(a != 0 && b != 0)
]

[(c)
  if(c > 10)
]
[(d)
  if (d < 20)
]

[(a,b,c,d)
  if(a != 0 && b != 0)
]
