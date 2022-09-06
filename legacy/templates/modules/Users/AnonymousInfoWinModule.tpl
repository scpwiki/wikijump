<div class="owindow" style="width: 60%">
	<div class="title">
		{t}Anonymous user info{/t}
	</div>
	<div class="content">
		<img style="float:right; padding: 2px 8px; background-color: #FFF;" src="/common--images/avatars/default/a48.png" alt="" />
		<h1>{t}Anonymous user{/t}</h1>

		<ul>
			<li>
				{t}IP address{/t}: <strong>{$ip}</strong> {if $privateIp}(<span id="private-range-help">{t}from a private IP range{/t} [?]</span>){/if}
			</li>

			{if $proxy}
				<li>
					{t}Proxy server{/t}: <strong>{$proxy}</strong>  <span id="proxy-help">[?]</span>
				</li>
			{/if}
		</ul>
		<p>
			{t 1=$SERVICE_NAME}This is an "Anonymous" user that does not use a valid account at %1.{/t}
		</p>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close window{/t}</a>
	</div>

	<div id="private-range-help-hovertip" style="display:none">
		{t}Private IP addresses are used mainly in local networks and are not visible
		(not routeable) from public internet nodes.<br/>
		If you see an IP address from a private range here it means a user is using a www proxy
		for web browsing.<br/>
		You cannot rely on a private IP address for unique computer identification.{/t}
	</div>
	{if $proxy}
	<div id="proxy-help-hovertip" style="display:none">
		{t}WWW Proxy.{/t}
	</div>
	{/if}
</div>
