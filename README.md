# panda-api | <a href="https://www.debugmyself.com/p/2020/1/24/panda_api_read_me/">中文文档</a>

Panda api makes it easier to build better api docs more quickly and easy for front end and back end.

Panda api encourages test driven development. it takes care of much of the hassle of web development between front end and back end, when you write done your api docs, you can focus on writing front end without needing to finish the backend. It’s free and open source.

Why Panda Api：

1. A better online read api docs.
2. Use json or json5 to write the api docs，eazy to lean and write.
3. Manage you api docs change as your code with git.
4. You can use Panda api as a back end api service with out backend develop.
5. Panda api takes test data helps developers auto test back end and front end
6. Suport define test case data
7. Mork data auto created
8. Environment route support, you can change the back end on panda api to development, test, production
9. Websocket support


## Getting started

### Install

- [Mac下的安装包](https://github.com/arlicle/panda-api/releases/download/v0.6/mac_install_0.6.zip)
- [Linux下的安装包](https://github.com/arlicle/panda-api/releases/download/v0.6/linux_install_0.6.zip)
- [Panda Api for Windows 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_windows_0.2.tar)

### A auth api docus

``` json5
{
    name:"Auth",
    desc:"user login and logout",
    order:1,
    api:
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
            code:{name:"response result code", type:"int", desc:"success is 1"},
            msg:{name:"response result message", type:"string", desc:""},
            token:{name:"login success, get a user token; login failed, no token", type:"string", required:false}
        },
        test_data:[
            {
                body:{username:"edison", password:"123"},
                response:{code:-1, msg:"password incorrect"}
            },
            {
                body:{username:"lily", password:"123"},
                response:{code:-2, msg:"username not exist"}
            },
            {
                body:{username:"root", password:"123"},
                response:{code:1, msg:"login success", token:"fjdlkfjlafjdlaj3jk2l4j"}
            },
            {
                body:{username:"lily"},
                response:{code:-1, msg:"password is required"}
            },
            {
                body:{password:"123"},
                response:{code:-1, msg:"username is required"}
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
            code:{name:"response result code", type:"int", desc:"success is 1"},
            msg:{name:"response result message", type:"string", desc:""}
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


#### Field options

Each field takes a set of field-specific arguments (documented in the body、query、response field reference). 

There’s also a set of common arguments available to all field types. All are optional. Here’s a quick summary of the most often-used ones:

##### name
the field name, default is the field

##### desc
the field description, defaulti is ""

##### type
default it string, the type can be: string, number, bool, object, array

##### default
the field default value

##### enum
enum value list , ex: enum:["a", "b", "c"]

##### required
If false, the field is optional. Default is true.


## Examples

1. [Basics](https://github.com/arlicle/panda-api-examples/tree/master/basics)
2. [Inherit from models](https://github.com/arlicle/panda-api-examples/tree/master/inherit_models)
3. [Global field settings](https://github.com/arlicle/panda-api-examples/tree/master/global_settings)



### Panda Api 如何使用
- [快速简单的写好第一个接口文档 使用说明(一)](https://www.debugmyself.com/p/2020/1/15/Panda-api%E4%BD%BF%E7%94%A8%E8%AF%B4%E6%98%8E/)
- [接口文档的高级配置 使用说明(二)](https://www.debugmyself.com/p/2020/1/15/Panda-api%E9%AB%98%E7%BA%A7%E4%BD%BF%E7%94%A8%E8%AF%B4%E6%98%8E/)
- [相关字段说明](https://www.debugmyself.com/p/2020/1/29/Panda-api%E5%AD%97%E6%AE%B5%E8%AF%B4%E6%98%8E/)
- [test_data使用说明](https://www.debugmyself.com/p/2020/1/27/Panda-api-test_data%E5%8C%B9%E9%85%8D%E8%AF%B4%E6%98%8E/)
- [Auth接口权限配置说明](https://www.debugmyself.com/p/2020/2/2/Panda-api接口权限配置说明/)
