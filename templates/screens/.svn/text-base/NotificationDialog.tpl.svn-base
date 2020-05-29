<div class="owindow" >
	<div class="title">
		{t}Account notifications{/t}
	</div>
	<div class="content">
	
		<h1>{t}Online account notifications{/t}</h1>
	
		<p>
			There {if $count == 1}is 1 new notification{else}are {$count} new notifications{/if} for you regarding your user account.
			To view the full list of notifications visit 
			<a href="http://{$URL_HOST}/account:you/start/notifications">my notifications</a>
			page.
		</p>
	
		<ul style="list-style: none; margin: 10px 0; padding: 0;">
			{foreach from=$notifications item=notification}
				<li style="margin: 5px 0" id="notification-{$notification->getNotificationId()}">
					<strong>{$notification->getTitle()}</strong>
					 - <span class="odate">{$notification->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span>
					<br/>
					{$notification->getBody()}
					{assign var=extra value=$notification->getExtra()}
					{if $notification->getUrls()}
						<br/>
						{t}Related links{/t}:
						{foreach from=$notification->getUrls() item=url}
						
							<a href="{$url[1]}">{$url[0]}</a>
						{/foreach}
					{/if}
				</li>
			{/foreach}
		</ul>
	
		{if $more !== null}
			<div>
				... and <a href="http://{$URL_HOST}/account:you/start/notifications">{$more} more new notifications</a>.
			</div>
		{/if}
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close window{/t}</a>
	</div>
</div>