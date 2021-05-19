{if !$delete}
	<h1>{t}Rename/move page{/t}</h1>

	<p>
		{t escape=no}The <em>rename</em> action will change the "unix name" of the page, i.e. the address
		via which the page is accessed.{/t}
	</p>
{else}
	<h1>{t}Delete page{/t}</h1>
	{if isset($canDelete)}
		<p>
			{t}
				You can delete the page by either move it to a "deleted" category or by just removing it
				from the Wiki - which is nonrecoverable and should be used with caution.
			{/t}
		</p>
		<table class="form">
			<tr>
				<td>
					{t}What to do?{/t}
				</td>
				<td>
					<input type="radio" name="how" value="rename" checked="checked" onclick="$('rename-option-delete').style.display='none';$('rename-option-rename').style.display='block'; OZONE.visuals.scrollTo('rename-option-rename');"> {t}just rename{/t}
					<br/>
					<input type="radio" name="how" value="delete" onclick="$('rename-option-delete').style.display='block';$('rename-option-rename').style.display='none'"> {t}delete completely{/t}
				</td>
			</tr>
		</table>
	{/if}
{/if}

<div id="rename-option-rename">
	{if isset($delete)}
		<p>
			{t}By preceding the page name with "deleted:" it can be moved to a different category (namespace).
			It is more or less equivalent to delete but no information is lost.{/t}
		</p>
	{/if}


	{if isset($isForum)}
		<div class="warning-block">
			<div class="title">Warning!</div>
			<p>
				This page might be important for proper functioning of the discussion forum.
			</p>
			<p>
				By renaming it, editing or deleting you could simply mess it up. These actions
				do not operate on forum elements such as threads, posts etc. but rather
				on particular Wiki pages that contain elements responsible for displaying
				forum elements.
			</p>
			<p>
				<b>Proceed only if you know what you are doing.</b>
			</p>
		</div>
	{/if}
	{if isset($isAdmin)}
		<div class="warning-block">
			<div class="title">Warning!</div>
			<p>
				This page might be important for proper functioning of the administrative stuff and
				most probably contains modules that allow you configuring and managing this Wiki.
			</p>
			<p>
				<b>Proceed only if you know what you are doing.</b>
			</p>
		</div>
	{/if}

	<p>
		{t}Attention should be also paid to the pages that depend on this one either by directly linking to
		it or by including it. Click below to see what pages depend on this one.{/t}
	</p>
	<p>
		<a id="rename-show-backlinks" href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.showBacklinks(event)">+ {t}show dependencies{/t}</a>
		<a id="rename-hide-backlinks" style="display:none" href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.hideBacklinks(event)">- {t}hide dependencies{/t}</a>
	</p>

	<div id="rename-backlinks-box" style="display:none"></div>

	<div id="rename-error-block" class="error-block" style="display: none"></div>
	<form onsubmit="Wikijump.modules.RenamePageModule.listeners.rename(event); return false;">
		<table class="form">
			<tr>
				<td>
					{t}Current page name{/t}:
				</td>
				<td>
					{$page->getUnixName()}
				</td>
			</tr>
			<tr>
				<td>
					{t}New page name{/t}:
				</td>
				<td>
					<input class="text" type="text" id="move-new-page-name" value="{$newName}" size="30"/>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="Wikijump.page.listeners.closeActionArea(event)"/>
			<input type="button" value="{t}rename/move{/t}" onclick="Wikijump.modules.RenamePageModule.listeners.rename(event)"/>
		</div>
	</form>
</div>

<div id="rename-option-delete" style="display: none">
	<p>
		This will remove the page completely and it will not be possible to recover the data. Are
		you sure you want to do this?
	</p>
	<form>
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="Wikijump.page.listeners.closeActionArea(event)"/>
			<input type="button" value="{t}delete{/t}" onclick="Wikijump.modules.RenamePageModule.listeners.deletePage(event)"/>
		</div>
	</form>
</div>
