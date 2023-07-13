
## **Nyaa-Notifications**

*stalk your favorite uploaders and don't miss the drama in the comments*

___

[As the RSS feed from nyaa doesn't provide comments and anything after the first page, I had to basically strip down the HTML to retrieve the data I want. Meaning: any changes to nyaa will likely brick this application and make it completely useless... until I fix it.](https://cdn.discordapp.com/attachments/768636792580341801/1126709125695410226/7937d3d659fb4af895ae.jpg)

___

### **Supported notification services:**
+ **SMTP/Mail:** (highlight because of newly found release)
![](https://i.imgur.com/XqPZMZt.png)
+ **Discord-Bot:**
![](https://i.imgur.com/EfM97GB.png)
* **Gotify:**
![](https://i.imgur.com/z6UOTAc.png)

**Notes:**
- Email & Discord: The avatar images of users aren't locally parsed. They are attached using their original src link from nyaa. This might be an issue for privacy concerned individuals.

___

### **Important information:**

#### Requirements for input domain:
* Must start at page 1.
* You have the option to search only the first page, or **ALL** of them. Think wisely as this could end up searching the entire website.

#### Config Notes:
* You can add multiple `Gotify` and `Email` modules as long as you **don't** change the order of the modules if your database has already been created.
* The `module_type` parameter specifies the behavior of the program, don't change it to something random.
* For the discord module, some parameters are not used but necessary to provide, just leave them in the file.
* Discord channels have be configured separately through the slash command framework.

#### Misc:
* Hard-coded rate-limits:
  * Nyaa - 2 seconds
  * Gotify - 2 seconds
  * Discord embeds - 1 second
  * Emails - none

___

### **Installation:**

Head over to the releases grab the binary and run it.

You should see a new folder `nyaa_notifications`.
There should be `config.toml` inside it with a little template.
Make sure you understand the structure of it and specify all the necessary and correct parameters.
There are no error checks for unsupplied parameters yet, so if you can't read, the binary might exit at any point in time.

___

Please contact me if you'd like to see a specific feature/change.
