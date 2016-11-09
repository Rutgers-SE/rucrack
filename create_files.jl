function create_ivfile(n)
    file = open("iv$n.txt", "w")
    content = rand(UInt8(0):UInt8(255), n)
    println(content)
    write(file, content)
    close(file)
end

create_ivfile(14)
create_ivfile(13)
create_ivfile(12)
create_ivfile(11)
