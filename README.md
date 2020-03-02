# panda-api | <a href="https://www.debugmyself.com/p/2020/1/24/panda_api_read_me/">中文文档</a>

大量使用说明，教程在中文文档中，请大家先看看中文文档和相关例子，忙于开发，等时间充足再写英文文档。

Panda api makes it easier to build better api docs more quickly and easy for front end and back end.

Panda api encourages test driven development. it takes care of much of the hassle of web development between front end and back end, when you write done your api docs, you can focus on writing front end without needing to develop the backend. It’s free and open source.

Why Panda Api：
1. A better online read api docs.
2. Use json5 to write the api docs，eazy to lean and write.
3. Manage you api docs change as your code with git.
4. You can use Panda api as a back end api service with out backend develop.
5. Panda api takes test data helps developers auto test back end and front end
6. Suport define test case data
7. Mork data auto created
8. Environment route support, you can change the back end on panda api to development, test, production
9. Websocket support



### Install

#### use installer (Recommended)

It looks like you’re running macOS, Linux, or another Unix-like OS. To download installer and install Panda api.

- [Mac installer](https://github.com/arlicle/panda-api/releases/latest)
- [Linux installer](https://github.com/arlicle/panda-api/releases/latest)
- [Windows installer](https://github.com/arlicle/panda-api/releases/latest)

#### Install by Source code
Get the latest development version
``` shell
git clone https://github.com/arlicle/panda-api.git
```
build and run panda api use `cargo`
``` shell
cargo run
```

Once Panda Api is installed (see Install above) do this in a terminal:
``` shell
panda --help
```
You should see the Panda Api command manual page printed to the terminal. This information includes command line options recognized by panda.


## Getting started

Let's build a simple project to get our feet wet. We'll create a new directory, say `my-project`, and a file in it, `auth.json5`:

``` shell
mkdir my-project
cd my-project
touch auth.json5
```

### write a panda api doc
Edit the file `auth.json5` with the following contents:

``` json5
{
    name:"Auth",
    desc:"user login and logout",
    order:1,
    apis:
    [{
        name:"user login",
        desc:"if user login success, will get a token",
        method: "POST",
        url:"/login/",
        body_mode:"json", // form-data, text, json, html, xml, javascript, binary
        body:{
            username:{name: "username"},
            password:{name: "password"}
        },
        response:{
            code:{name:"response result code", type:"int", desc:"success is 1", enum:[-1, 1]},
            msg:{name:"response result message", type:"string"},
            token:{name:"login success, get a user token; login failed, no token", type:"string", required:false}
        },
        test_data:[
            {
                body:{username:"edison", password:"123"},
                response:{code:-1, msg:"password incorrect"}
            },
            {
                body:{username:"lily", password:"123"},
                response:{code:-1, msg:"username not exist"}
            },
            {
                body:{username:"root", password:"123"},
                response:{code:1, msg:"login success", token:"fjdlkfjlafjdlaj3jk2l4j"}
            }
        ]
    },
    {
        name:"user logout",
        method:"GET",
        url:"/logout/",
        query:{
            id:{name:"user id"},
            username:{}
        },
        response:{
            code:{name:"response result code", type:"int", desc:"success is 1", enum:[-1, 1]},
            msg:{name:"response result message", type:"string"}
        },
        test_data:[
            {
                query:{id:1, username:"root"},
                response:{code:1, msg:"logout success"}
            },
            {
                response:{code:-1, msg:"error"}
            },
            {
                query:{id:3, username:"lily"},
                response:{code:-1, msg:"username and id not match"}
            }
        ]
    }
]}
```


run command `panda` in the `my-project`
``` shell
panda
```
You should see run info:
``` shell
 INFO  actix_server::builder > Starting 8 workers
 INFO  actix_server::builder > Starting "actix-web-service-127.0.0.1:9000" service on 127.0.0.1:9000
```
### view online api docs
Now we can view the api docs online `http:://127.0.0.1:9000` or `http://localhost:9000`

Notice if you get a error
``` shell
Error: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
```
It's mean the port 9000 is in use, you need to change another one.
```
panda -p 9001
```
### request the api
When the panda is running, we can request api in the docs without write a code of backend.

we request the api `/login/` with `test_data` in the docs `auth.json5`.

1th:
```.language-shell
curl localhost:9000/login/ -X POST -H "Content-Type:application/json" -d '{"username":"edison","password":"123"}'
// you will get response
{"code":-1,"msg":"password incorrect"}
```

2th:
```.language-shell
curl localhost:9000/login/ -X POST -H "Content-Type:application/json" -d '{"username":"lily","password":"123"}'
// you will get response
{"code":-1,"msg":"username not exist"}
```

3th:
```.language-shell
curl localhost:9000/login/ -X POST -H "Content-Type:application/json" -d '{"username":"root","password":"123"}'
// you will get response
{"code":1,"msg":"login success"}
```

If you request data not defined in the `test_data`, You will get a mock response

```.language-shell
curl localhost:9000/login/ -X POST -H "Content-Type:application/json" -d '{"username":"hello","password":"123"}'
// you will get response like this
{"code":1,"msg":"SqM!3Mky@)q1O","token":"OkkdvtKKl(htx#KU6"}
```

Pretty simple, right?

mock options can help the mock data more like the production environment, update api `/login/` `response` define:
```
...
response:{
    code:{name:"response result code", type:"int", desc:"success is 1", enum:[-1, 1]},
    msg:{name:"response result message", type:"sentence"}, // update type string to sentence
    token:{name:"login success, get a user token; login failed, no token", type:"string", required:false, length:64} // set the token length:64
},
...
```
request data not defined in the `test_data` again:
```.language-shell
curl localhost:9000/login/ -X POST -H "Content-Type:application/json" -d '{"username":"hello","password":"123"}'
// you will get response like this
{"code":1,"msg":"Qphxw ddfcvpy odpi ikdd, ","token":"PRL3%S%Uc&33X%HB*Yflc3qQt(LnC)cf6^0w357F07r3xUyafsvS#mr8BZw6UrMo"}
```

more field options in here: [https://www.debugmyself.com/p/2020/1/29/Panda-api%E5%AD%97%E6%AE%B5%E8%AF%B4%E6%98%8E/](https://www.debugmyself.com/p/2020/1/29/Panda-api%E5%AD%97%E6%AE%B5%E8%AF%B4%E6%98%8E/)


### array and object field
```.language-json5
response:{
    total_page: {name:"total page", type:"number"},
    current_page: {name:"current page num", type:"number"},
    result:
        [{
            id:{name:"Article ID", type:"PosInt"},
            title:{name:"Article title"},
            category:{
                id:{name:"category id"},
                name:{name:"category name"}
            },
            author_name:{name:"Author name"},
            tags:[{
                id:{name:"Tag id", type:"PosInt"},
                name:{name:"tag name"}
            }],
            created:{name:"article created time", type:"timestamp"}
        }]
}
```


### inherit model
``` shell
mkdir _data
cd _data
touch models.json5
```

``` json5
// _datat/models.json5
{
    Article:{
        id:{name:"Article ID", type:"PosInt"},
        title:{name:"Article Title"},
        category:{
            id:{name:"Category ID",},
            name:{name:"Category Name"}
        },
        author_name:{name:"Author name"},
        tags:[{
            id:{name:"Tag id", type:"PosInt"},
            name:{name:"Tag name"}
        }],
        created:{name:"article created time", type:"timestamp"}
    }
}
```

``` json5
body: {
    $ref:"./_data/models.json5:Article",
    $exclude:["created", "category/name", "tags/0/name"],
    id:{name:"Article ID", type:"PosInt", required:false},
}
```

``` json5
response: {
    $ref:"./_data/models.json5:Article",
    $exclude:["created", "category/name", "tags/0/name"],
    id:{name:"Article ID", type:"PosInt", required:false},
}
```


## Examples

1. [Basics](https://github.com/arlicle/panda-api-examples/tree/master/basics)
2. [Inherit from models](https://github.com/arlicle/panda-api-examples/tree/master/inherit_models)
3. [Global field settings](https://github.com/arlicle/panda-api-examples/tree/master/global_settings)



### Panda Api 如何使用
- [快速简单的写好第一个接口文档 使用说明(一)](https://www.debugmyself.com/p/2020/1/15/Panda-api-how-to-use/)
- [快速开发的视频教程](https://www.bilibili.com/video/av88926940?p=2)
- [接口文档的高级配置 使用说明(二)](https://www.debugmyself.com/p/2020/1/15/Panda-api-how-to-use2/)
- [相关字段说明](https://www.debugmyself.com/p/2020/1/29/panda-api-field/)
- [test_data使用说明](https://www.debugmyself.com/p/2020/1/27/panda-api-test_data/)
- [Auth接口权限配置说明](https://www.debugmyself.com/p/2020/2/2/Panda-api-auth/)
- [settings配置说明](https://www.debugmyself.com/p/2020/2/16/panda_api_settings/)



