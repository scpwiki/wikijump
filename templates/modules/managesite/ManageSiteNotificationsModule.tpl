<h1>Notifications</h1>

<p>
	Below is the list of all notifications related to your account.
</p>	

{*<div id="notifications-area"><div class="wait-block">please wait...</div></div>*}
<div id="notifications-area">
	{module name="managesite/ManageSiteNotificationsListModule"}
</div>


<h2>Notifications via RSS feed</h2>

<p>
	Notifications related to this site are also available via a password-protected 
	RSS 2.0 feed:
</p>
<p style="text-align: center">
	<a href="http://{$site->getDomain()}/feed/admin.xml">http://{$site->getDomain()}/feed/admin.xml</a>
</p>
<ul>
	<li>
		In order to view the feed you must authenticate via Basic HTTP Authentication
		mechanism. Many of the news aggregators support that method.
	</li>
	<li>
		Use the following data for authentication:
		<ul>
			<li>username: {$feedUsername}</li>
			<li>password: {$feedPassword}</li>
		</ul>
	</li>
	<li>
		The password above is a hashed version of your login password. It can be used only for
		your feed access and not for login - so in principle it should be safe to use.
	</li>
	<li>
		Only the Site Administrators are allowed to view this feed.
	</li>
</ul>