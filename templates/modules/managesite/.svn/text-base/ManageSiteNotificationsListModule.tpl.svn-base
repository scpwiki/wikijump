<div style="text-align:center">
{pager jsfunction="WIKIDOT.modules.ManageSiteNotificationsModule.listeners.loadList(event,#)" total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page} 
</div>

{if $notificationsCount>0}
	
	<ul style="list-style: none; margin: 10px 0; padding: 0;">
		{foreach from=$notifications item=notification}
			<li style="margin: 5px 0" id="notification-{$notification->getNotificationId()}">
				<strong>{$notification->getTitle()}</strong>
				(<span class="odate">{$notification->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>)
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
{else}
	<p>
		{t}Sorry, no notifications (yet).{/t}
	</p>
{/if}