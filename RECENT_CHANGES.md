* Nyaa runs on python, so the comment amount displayed on the torrent page might not match the actual amount of torrents on the page (yet).
  * Avoid problems when this happens.
* Turn off comment notifications if no notification methods are activated at all.
* Nyaa is now saving the default avatar on their own page, so the `src` value isn't a valid link anymore
  * this should also fix most messages being sent twice from the discord bot
* Hopefully fixed SQL calls with strings containing double quotes.
* Fixed error when no notification service is activated (don't retrieve comments)
