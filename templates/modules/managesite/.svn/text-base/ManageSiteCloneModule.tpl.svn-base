<h1>Clone the Site</h1>

<div id="sm-clone-block">
<p>
	Using this feature you are able to clone the whole wiki. A new wiki with content copied from this one
	will be created with the new web address.
</p>

<h2>What will be cloned?</h2>

<p>Will be cloned</p>
<ul>
	<li>all pages and their content</li>
	<li>page tags</li>
	<li>category settings</li>
	<li>permission settings</li>
	<li>themes (including custom themes)</li>
	<li>forum structure (groups and categories)</li>
</ul>

<p>Will not be cloned</p>

<ul>
	<li>page edit history (revisions)</li>
	<li>forum threads and posts</li>
	<li>members</li>
	<li>custom domain settings</li>
</ul>

<h2>Override</h2>

<p>
	Any option you provide below will override the copied property of the existing Site. Of course, 
	a unique web address (unix name) needs to be provided.
</p>

<div class="error-block" id="clone-site-form-errors" style="display: none"></div>

<form id="clone-site-form">
	<table class="form">
		<tr>
			<td>
				{t}Site name{/t}:
			</td>
			<td>
				<input class="text" type="text" id="new-site-name" name="name" size="30" value="{$site->getName()|escape}" />
				<div class="sub">
					{t}Appears on the top-left corner of your Wikidot site.{/t}
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}Tagline{/t}:
			</td>
			<td>
				<input class="text" type="text" name="tagline" size="30" value="{$site->getSubtitle()|escape}"/>
				<div class="sub">
					{t}Appears beneath the name.{/t}
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}Web address{/t}:
			</td>
			<td>
				<input class="text" type="text" id="new-site-unixname" name="unixname" size="20" style="text-align: right" value=""/>.{$URL_DOMAIN}
				<div class="sub">
					{t}Only alphanumeric [a-z0-9] and "-" (dash) characters allowed.{/t}
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}Description{/t}:
			</td>
			<td>
				<textarea class="textarea" name="description" cols="40" rows="5">{$site->getDescription()|escape}</textarea>
			</td>
		</tr>
		<tr>
			<td>
				{t}Private site?{/t}
			</td>
			<td>
				<input type="checkbox" name="private" class="checkbox" {if $site->getPrivate()}checked="checked"{/if}>
				<div class="sub">
					{t}If you check this, the site is visible only to its members.{/t}
				</div>
			</td>		
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}Cancel{/t}" onclick="WIKIDOT.modules.ManageSiteCloneModule.listeners.cancel(event)"/>
		<input type="button" value="{t}Clone this Site{/t}" onclick="WIKIDOT.modules.ManageSiteCloneModule.listeners.cloneSite(event)"/>
	</div>
</form>

</div>