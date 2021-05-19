<h1>Blocked users</h1>

<p>
	As an administrator of the site you can block certain users from modyfying
	contents of the site. Such users will only be able to view contents of the site
	but not to alter it in any way.
</p>

<h2>List of blocked users</h2>
{if isset($blocks)}
	<ul style="list-style: none">
		{foreach from=$blocks item=block}
			<li style="margin: 0.2em 0">
				{printuser user=$block->getUser() image="true"}
				<br/>
				<div style="position: absolute; margin-left: 30em">
					<a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserBlocksModule.listeners.deleteBlock(event, {$block->getUserId()}, '{$block->getUser()->username|escape}')">delete block</a>
				</div>
				blocked on: <span class="odate">{$block->getDateBlocked()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
				{if $block->getReason() && $block->getReason() != ''}
				<br/>reason: {$block->getReason()|escape}
				{/if}

			</li>
		{/foreach}
	</ul>
{else}
there are no blocked users.
{/if}
<div id="show-add-block-button">
	<a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserBlocksModule.listeners.showAddForm(event)">+ add user to blocklist</a>
</div>

<div id="add-block-user-div" style="display: none">
	<h2>Add user to block list</h2>

	<table class="form">
		<tr>
			<td>
				User to be blocked:
			</td>
			<td>
				<div id="select-user-div">
					<div class="sub">
						type the Wikijump user name below
					</div>
					<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">
						<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>
						<div id="user-lookup-list" class="autocomplete-list"></div>
					</div>
				</div>
				<div id="selected-user-div" style="display: none">
					<span id="selected-user-rendered"></span> (<a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserBlocksModule.listeners.changeUser(event)">change</a>)
				</div>
			</td>
		</tr>
		<tr>
			<td>
				Reason:
			</td>
			<td>
				<textarea id="user-block-reason" cols="40" rows="5"></textarea>
				<div class="sub">
					<span id="reason-char-left"></span> characters left.
				</div>
			</td>
		</tr>
	</table>


	<div class="buttons">
		<input type="button" value="cancel" onclick="Wikijump.modules.ManageSiteUserBlocksModule.listeners.cancelAdd(event)"/>
		<input type="button" value="block user" onclick="Wikijump.modules.ManageSiteUserBlocksModule.listeners.blockUser(event)"/>
	</div>
</div>
