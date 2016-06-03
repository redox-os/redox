#!/bin/sh
if a == a
  echo true a == a

  if b != b
    echo true b != b
  else
    echo false b != b

    if 3 '>' 2
      echo true 3 '>' 2
    else
      echo false 3 '>' 2
    end
  end
else
  echo false a == a
end
