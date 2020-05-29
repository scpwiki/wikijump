<h1>{t}Public or private{/t}</h1>

<p>
	By default all Sites hosted by {$SERVICE_NAME} are public. If you however choose this Site
	to be private, only its members + explicitely permitted users will be able to view its contents and display pages.
	You should also check out <em>Permissions</em> to set who can edit it.
</p>

<p style="font-weight: bold">
	If you choose the site to be private it is limited to {$settings->getMaxPrivateMembers()} members + extra {$settings->getMaxPrivateViewers()}
	users with access permission. Please contact <a href="mailto:{$SUPPORT_EMAIL}">{$SUPPORT_EMAIL}</a>
	to learn more how to increase these limits.
</p>
<p style="font-weight: bold">
	The above limits apply only if the site is private. Public sites have no limit on number of users.
</p>

<form id="sm-private-form">
	<table class="form">
		<tr>
			<td style="width: 20em">
				{t}Make this Site private{/t}:
			</td>
			<td>
				<input type="checkbox" class="checkbox" name="private" {if $site->getPrivate()}checked="checked"{/if}/>
			</td>
		</tr>
		<tr>
			<td>
				{t}Default landing page for unauthorized visitors{/t}:
			</td>
			<td>
				<div class="autocomplete-container" style="width: 20em">
					<input type="text" id="sm-private-land" class="autocomplete-input text" name="landingPage" size="35" value="{$settings->getPrivateLandingPage()}"/>
					<div id="sm-private-land-list" class="autocomplete-list"></div>
				</div>
				<div class="sub">
					{t}E.g. system:join.{/t}
				</div>
			</td>
		</tr>
		<tr>
			<td style="width: 20em">
				{t}Hide nav elements for unauthorized users{/t}:
			</td>
			<td>
				<input type="checkbox" class="checkbox" name="hideNav" {if $settings->getHideNavigationUnauthorized()}checked="checked"{/if}/>
				<div class="sub">
					If checked - unauthorized users will not be able to see <br/>
					neither your side bar nor top bar navigation elements. <br/>
					See a note below.
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}Extra access{/t}:
			</td>
			<td>
				<div id="select-user-div" style="text-align: right">
					<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">
						<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>
						<div id="user-lookup-list" class="autocomplete-list"></div>
					</div>
				</div>	
				<div class="sub">
					Specify users that will be granted access to the Site <br/>
					without having to become Site Members.<br/>
					Type the user name and hit enter.
				</div>	
			</td>
		</tr>
		<tr>
			<td>
				Current extra access list:
			</td>
			<td>&nbsp;</td>
		</tr>
	</table>
	<div style="text-align: center" id="extra-viewers-display-list"></div>
	
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ManageSitePrivateSettingsModule.listeners.save(event)"/>
	</div>
</form>

<h2>{t}Tips{/t}</h2>
<p>
	{t}The landing page for unauthorized visitors should at least explain why the site is private 
	and how to get access to it. But this is of course up to you.{/t}
</p>
<p>
	You can better manage the style and look of the welcome page for unauthorized visitors by
	creating a welcome page	in a different category 
	(e.g. <tt>unauthorized:welcome</tt>) and disable nav elements for it in the 
	Appearance menu.
</p>
<p>
	If your Site is already private and you want to invite new members, you can:
</p>
<ul>
	<li>
		<a href="javascript:;" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-email-invitations')">{t}Send email invitations{/t}</a>
		- does not matter if they already have an account at {$SERVICE_NAME} or not,
	</li>
	<li>
		<a href="javascript:;" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-members-invite')">{t}Invite Wikidot.com users{/t}</a>
		by sending them an "internal" invitation,
	</li>
	<li>
		Prepare your "landing page" so that it accepts either users' applications for membership
		(module MembershipApply) or enables becoming a member by providing a secret password (module 
		MembershipByPassword - see <a href="{$URL_DOCS}">documentation</a>).
	</li>
</ul>

<div id="viewers-list-div" style="display: none">
	{foreach from=$viewers item=user}
		<div id="viewer-entry-{$user->getUserId()}">{$user->getNickName()|escape}</div>;
	{/foreach}
</div>