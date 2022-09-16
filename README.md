## **Nyaa-Notifications**

*stalk your favourite uploaders and don't miss the drama in the comments*

___

As the rss feed from nyaa doesn't provide comments and anything after the first page, I had to basically strip down the html to retrieve the data I want. Meaning: any changes to nyaa will likely brick this application and make it completely useless... until I fix it.

___

### **Supported notification services:**
+ **SMTP/Mail:** (highlight because of newly found release)
![](https://i.imgur.com/XqPZMZt.png)
+ **Discord-Bot:**
![](https://i.imgur.com/KtzIDv6.png)
* **Gotify:**
![](https://i.imgur.com/9UzbkyP.png)

**Notes:**
- The avatar images of users aren't locally parsed. They are attached using their original gravatar.com link from nyaa. This might be an issue for privacy concerend individuals.

___

### **Important information:**

#### Requirements for input domain:
* Must start at page 1.
* If the input url contains search patterns (aside from "newest"), the script will download all pages to find a new release. This can get your ip **banned** if you input the wrong url. (`complete_result = false`: limits everything to the first page)

#### Config notes:
* To add multiple discord-notification channels, you can just continue the `channel_id` list.
* On the first run, I'd highly suggest you to keep all notification services deactivated, so you don't get spammed with outdated news.

#### Misc:
* All requests are executed two seconds from each other.

___

### **Installation:**

Head over to the releases grab the binary and run it.
The output on the first run will tell you what to do next.

___

Please tell me if you'd like to see a specific feature.
