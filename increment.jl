i(n::Number)=UInt8(n)
i(ns::Number...)=map(i, collect(ns))

function inc(values::Vector{UInt8})
  idx = 1
  while true
    idx > length(values) && break

    shouldinc, values[idx] = inc(values[idx])
    if shouldinc
      idx = idx + 1
    else
      break
    end
  end
  values
end

function inc(value::UInt8)
  tmp = value
  value = value + UInt8(1)
  if value < tmp
    (true, value)
  else
    (false, value)
  end
end

function test_inc()
  values = i(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15)
  while values[end] != 0
    values = inc(values)
    println(values)
  end
end
