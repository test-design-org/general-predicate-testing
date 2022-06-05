import React, { useEffect } from 'react';
import ManualTester from './components/ManualTester';

import './App.scss';
import { parseGpt, traverseAST } from '@testing-repo/gpt-compiler';

const test = String.raw`
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
`;

const test2 = String.raw`
[

  var VIP: bool
  var price: num
  var second_hand_price: num

  if(VIP = true &&  price <50) {
    if(second_hand_price = 2)
    // if(second_hand_price = 3)
  }
  if(VIP = false &&  price >=50)
  if(VIP = true &&  price >=50)
  if(price >30 && second_hand_price >60)

  ]
`;

function App() {
  useEffect(() => {
    // console.log(JSON.stringify(parseGpt(test)));
    console.log(JSON.stringify(parseGpt(test2)));
    console.log(JSON.stringify(traverseAST(parseGpt(test2))));
  }, []);
  return (
    <>
      <a
        href="https://test-design.org/practical-exercises/"
        className="backLink"
      >
        Back to Test Design Exercises
      </a>
      <ManualTester />
    </>
  );
}

export default App;
