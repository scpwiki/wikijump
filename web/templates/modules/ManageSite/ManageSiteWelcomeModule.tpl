<h1>{t}Welcome to the site manager{/t}!</h1>

<p>
	{t}This is the administrative panel for your Site.{/t}

	{t escape=no}But remember: <b>With Great Power Comes Great Responsibility</b>.{/t}
</p>

{if isset($tips)}

	<h2>{t}A few tips for your Site{/t}</h2>

	<ul>
		{if $tips.forum}
			<li>
				<strong>Start a forum</strong><br/>
				The simplest way to add interactivity to your Site is to start a discussion forum.
				Click on <em>Forum &amp; discussion</em> &gt;&gt; <em>Settings</em> to learn more.<br/>
			</li>
		{/if}
		{if $tips.tags}
			<li>
				<strong>Use site tags</strong><br/>
				If your Site already has some interesting content you should describe your Site by appropriate Tags
				- it will be easier to find for others. Go to
				<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-general')">General settings</a>.
			</li>
		{/if}
		{if $tips.invite}
			<li>
				<strong>{t}Invite your friends or coworkers!{/t}</strong><br/>
				{t}It is easy to invite new people to join your site as members
				(if you need members at all).
				You can just send them emails with special invitations.{/t}<br/>
				<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-email-invitations')">{t}Send email invitations now!{/t}</a>.
			</li>
			<li>
				Or even better - <a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-users-email-invitations')">you can allow your members to invite their friends</a> if you want the
				community to grow quickly!
			</li>
		{/if}

	</ul>

{/if}

<h2>Useful links</h2>

<ul>
	<li>
		<a href="{$URL_DOCS}" target="_blank">Help &amp; Documentation</a> if you want to learn more or need help.
	</li>
</ul>
