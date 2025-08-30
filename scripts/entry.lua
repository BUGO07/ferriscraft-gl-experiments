x = 5

function startup()
    print("startup from lua")
end

function update()
    x = x + 0.01
    print("update from lua | number - "..x)
end