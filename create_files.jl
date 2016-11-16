function create_ivfile(n)
    file = open("iv$n.txt", "w")
    write(file, rand(UInt8(0):UInt8(255), n))
    close(file)
end

create_ivfile(14)
create_ivfile(13)
create_ivfile(12)
create_ivfile(11)
