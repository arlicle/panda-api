# panda-api | <a href="https://www.debugmyself.com/p/2020/1/24/Panda-api%E4%B8%AD%E6%96%87%E8%AF%B4%E6%98%8E/">中文文档</a>

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

[Panda Api for Mac 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_mac_0.2.tar)  
[Panda Api for Linux 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_linux_0.2.tar)  
[Panda Api for Windows 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_windows_0.2.tar)

### A auth api docus

``` json5
{
    name:"Auth",
    desc:"user login and logout",
    order:1,
    api:
    [{
        name:"用户登录",
        desc:"用户登录成功，接口会返回一个token",
        method: "POST",
        url:"/login/",
        body_mode:"json", // form-data, text, json, html, xml, javascript, binary
        body:{
            username:{name:"用户名"},
            password:{name:"用户登录密码"}
        },
        response:{
            code:{name:"返回结果的代码", type:"int", desc:"登录成功只返回1"},
            msg:{name:"登录成功返回消息", type:"string", desc:"通常返回都是空"},
            token:{name:"登录成功返回的用户token", type:"string", required:false}
        },
        test_data:[
            {
                body:{username:"edison", password:"123"},
                response:{code:-1, msg:"密码输入不正确"}
            },
            {
                body:{username:"lily", password:"123"},
                response:{code:-2, msg:"用户名不存在"}
            },
            {
                body:{username:"root", password:"123"},
                response:{code:1, msg:"登录成功", token:"fjdlkfjlafjdlaj3jk2l4j"}
            },
            {
                body:{username:"lily"},
                response:{code:-1, msg:"密码是必填的"}
            },
            {
                body:{password:"123"},
                response:{code:-1, msg:"用户名是必填的"}
            }
        ]
    },
    {
        name:"用户退出登录",
        method:"GET",
        url:"/logout/",
        query:{
            id:{name:"用户id"},
            username:{name:"用户名"}
        },
        response:{
            code:{name:"返回结果的代码", type:"int", desc:"退出成功只返回1"},
            msg:{name:"退出操作返回消息", type:"string", desc:"通常返回都是空"}
        },
        test_data:[
            {
                query:{id:1, username:"root"},
                response:{code:1, msg:"退出成功"}
            },
            {
                response:{code:-1, msg:"非法操作"}
            },
            {
                query:{id:3, username:"lily"},
                response:{code:-1, msg:"用户名和id不匹配"}
            }
        ]
    }
]}
```


#### Field options

Each field takes a set of field-specific arguments (documented in the body、query、response field reference). 

There’s also a set of common arguments available to all field types. All are optional. Here’s a quick summary of the most often-used ones:

##### name
the field name

##### desc
the field description

##### type
default it string, the type can be: string, number, bool, object, array

##### default
the field default value

##### enum
enum value list , ex: enum:["a", "b", "c"]

##### required
If False, the field is optional. Default is True.

##### 


### 带有样例的使用包下载 
[Panda Api for Mac 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_mac_0.2.tar)  
[Panda Api for Linux 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_linux_0.2.tar)  
[Panda Api for Windows 0.2](https://github.com/arlicle/panda-api/releases/download/0.2/panda-api_windows_0.2.tar)

### 使用文档
[Panda Api使用说明(一)](https://www.debugmyself.com/p/2020/1/15/Panda-api%E4%BD%BF%E7%94%A8%E8%AF%B4%E6%98%8E/)  
[Panda Api使用说明(二)](https://www.debugmyself.com/p/2020/1/15/Panda-api%E9%AB%98%E7%BA%A7%E4%BD%BF%E7%94%A8%E8%AF%B4%E6%98%8E/)  
[Mac下安装配置](https://www.debugmyself.com/p/2020/1/17/Mac%E4%B8%8B%E5%AE%89%E8%A3%85Panda-Api/)  
[Windows下安装配置](https://www.debugmyself.com/p/2020/1/18/Windows%E4%B8%8B%E5%AE%89%E8%A3%85Panda-Api/)  
