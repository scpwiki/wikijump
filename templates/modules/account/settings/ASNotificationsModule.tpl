<h1><a href="javascript:;" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')">Account settings</a> / Notifications</h1>

<p>
	When an event important to you takes place
	you can be informed about it in a few different ways:
</p>
{*
<form>
	<table class="form grid">
		<tr>
			<td rowspan="2" style="vertical-align: bottom">
				Event type
			</td>
			<td colspan="3" style="text-align: center;">
				Notification channel
			</td>
		</tr>
		<tr>
			<td>
				online
			</td>
			<td>
				email
			</td>
			<td>
				private feed
			</td>
		</tr>
		<tr>
			<td>
				new private message
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-o"/>
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-e"/>
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-f"/>
			</td>
		</tr>
		<tr>
			<td>
				new private message
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-o"/>
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-e"/>
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="m-f"/>
			</td>
		</tr>

	</table>
	
	<div class="buttons">
		<input type="button" value="cancel" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')"/>
		<input type="button" value="save" onclick="WIKIDOT.modules.ASNotificationsModule.listeners.save(event)"/>
	</div>
</form>
*}
<p>
	Currently there are no options to configure what type of notifications 
	fall into which notification channel - this will be developed later.
</p>

<h2>Online notifications</h2>

<p>
	From time to time a pop-up window with the new notifications appears to inform you about
	events related to your account. This happens only when you are logged in and browse
	any of the Wikidot Sites.
</p>

<h2>Email notifications</h2>
<p>
	It is possible to receive a daily digest email with unread notifications. Such emails are
	generated sometime around 5AM UTC and contain information about events related to your account
	(including new unread private messages).
</p>
<table class="form">
	<tr>
		<td>
			Receive daily digest?
		</td>
		<td>
			<input type="checkbox" class="checkbox" id="as-receive-digest" {if $settings->getReceiveDigest()}checked="checked"{/if}/>
		</td>
		<td>
			<input type="button" class="button" value="apply setting" onclick="WIKIDOT.modules.ASNotificationsModule.listeners.saveReceiveDigest(event)"/>
		</td>
	</tr>

</table>

<h2>{$SERVICE_NAME} Newsletter</h2>
<p>
	From time to time we send a newsletter that cover the most important changes in the 
	{$SERVICE_NAME} Services. We certainly do not send anything that would be considered "spam".
	It is highly recommended to accept our Newsletter.
</p>

<table class="form">
	<tr>
		<td>
			Receive Wikidot Newsletter?
		</td>
		<td>
			<input type="checkbox" class="checkbox" id="as-receive-newsletter"  {if $settings->getReceiveNewsletter()}checked="checked"{/if}/>
		</td>
		<td>
			<input type="button" class="button" value="apply setting" onclick="WIKIDOT.modules.ASNotificationsModule.listeners.saveReceiveNewsletter(event)"/>
		</td>
	</tr>

</table>

<h2>Notifications via RSS feed</h2>

<p>
	Notifications related to your account are also available via a personalized 
	RSS 2.0 feed:
</p>
<p style="text-align: center">
	<a href="http://{$URL_HOST}/feed/account/notifications.xml">http://{$URL_HOST}/feed/account/notifications.xml</a>
</p>
<ul>
	<li>
		In order to view the feed you must authenticate via Basic HTTP Authentication
		mechanism. Many of the news aggregators support that method.
		<br/>
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
		your feed access and not for login - so in principle you can safely use it.
	</li>
</ul>