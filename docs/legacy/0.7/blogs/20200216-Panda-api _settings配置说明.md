---
title: "Panda Api _settings配置说明"
post-date: "2020-02-16 14:21"
slug: "panda_api_settings"
---

Panda Api `_settings.json5` 文件包含了所有的基础设置.

这是一个默认的基础配置:
```.language-json5
// _settings.json5
{
  project_name: "Panda Api",
  project_desc: "Panda Api is a simple api manage tool"
}
```
`project_name`配置项目名称, `project_desc`:配置项目简介, 如果需要详细的大篇幅的项目简介, 可以放到`README.md`文件中, Panda Api会自动进行加载.

### 全局字段属性配置
我们可以为api接口字段的一些公共属性进行全局的设置, 比如,如果大部分接口都是需要`auth`, 那么我们就可以在全局变量中设置`auth`为`true`, 如果所有接口的`response`都有`code`和`msg`,那么我们就可以在全局`response`配置上`code`和`msg`

```.language-json5
{
  project_name: "Panda Api",
  project_desc: "Panda Api is a simple api manage tool",
  global: {
    apis:{
      auth: true,
      response:{
        code:{name:"返回的状态码", type:"number"},
        msg:{name:"返回的状态说明"}
      }
    },
  }
}
```

一旦配置后,所有有的接口就会默认继承有`auth:true`, 所有接口的`response`就自动有`code:{name:"返回的状态码", type:"number"}`和`msg:{name:"返回的状态说明"}`,
### 重写继承的属性:
虽然我们所有接口都会继承有`global`中设置的属性,我们还是可以根据需要重写这些属性,一旦在自己接口中重新定义属性`auth`或者`response`的`code`或`msg`,属性就会以接口定义的为准, 例如:

```.language-json5
// auth.json5
{
    url:"/login/",
    method:"POST",
    auth:false,
    body_mode:"json",
    body:{
        ...
    },
    response:{
        result:{...}
    }
}
```
这个例子中,接口'/login/'原本继承了全局属性`auth:true`,但是在接口重新定义了`auth:false`,那么最终结果以接口定义为准.
`response`中定义了一个`result`属性,未重写`code`和`auth`, 所以最终结果为:
```.language-json5
// auth.json5
{
    url:"/login/",
    method:"POST",
    auth:false,
    body_mode:"json",
    body:{
        ...
    },
    response:{
        code:{name:"返回的状态码", type:"number"},
        msg:{name:"返回的状态说明"},
        result:{...}
    }
}
```

### 删除继承的属性
我们可以把不要的属性进行删除,例如:
```.language-json5
// auth.json5
{
    url:"/login/",
    method:"POST",
    auth:false,
    body_mode:"json",
    body:{
        ...
    },
    response:{
        code:{$del:true},
        result:{...}
    }
}
```
我们设置了`response`的`code`属性为删除,那么就不会继承这个属性. 最终结果将为:
```.language-json5
// auth.json5
{
    url:"/login/",
    method:"POST",
    auth:false,
    body_mode:"json",
    body:{
        ...
    },
    response:{
        msg:{name:"返回的状态说明"},
        result:{...}
    }
}
```
可以看到,`code`字段没有了,


### 设置全局base_path
我们可以为所有接口的`url`设置一个`base path`, 例如有的团队的开发习惯,喜欢把版本标记在url地址上,例如`/v1/login/`, `/v3/login/`, 或者有的是子项目,所有接口的`url`都会有一个子项目的地址`/sub/login/`, `/sub/article/list/`, 遇到这样的情况,我们就可以设置全局`base_path`,

```.language-json5
{
    project_name: "Panda Api",
    project_desc: "Panda Api is a simple api manage tool",
    global: {
        apis:{
            base_path:"/v2/", // 全局 base path
            auth: true,
            response:{
                code:{name:"返回的状态码", type:"number"},
                msg:{name:"返回的状态说明"}
            }
        },
    }
}
```
我们在全局`global`中设置了`base_path`为`/v2/`, 所以我们所有接口文档中的`url`在文档输出时全部都会加上`/v2/`前缀, 比如用户登录的接口地址为`/login/`会自动变成`/v2/login/`, 所以我们请求的地址也要相应的改变为`/v2/login/`.

如果我们不需要,同样可以在接口中重写`base_path`属性.

### 配置:开发服务器 测试服务器 正式服务器
我们可以配置我们开发的各类服务器环境,然后可以使用Panda Api自动进行各类接口的测试, 还是可以在本地开发的时候, 路由请求到对应服务器上.
```.language-json5
servers:{
    dev:{
        name:"开发服务器",
        desc:"",
        url:"http://127.0.0.1:9000",
    },
    test:{
        name:"测试服务器",
        url:"http://www.c.com",
    },
    pro:{
        name:"正式服务器",
        url:"http://www.b.com",
    }
}
```
