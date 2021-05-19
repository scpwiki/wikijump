<h1>Delete the whole Site</h1>

<p>
	Yes, you <b>can</b> delete the whole Site. However even if you delete it, it will not
	be wiped out from our database but rather you will be able to <b>undo</b> the operation.
</p>
<p>
	This action is available only to the person who actually
	started the Site (the founder).
</p>

<p>
	We highly discourage anyone from deleting a website at Wikijump especially if:
</p>
<ul>
	<li>
		your site is publicly recognizable and potentially useful,
	</li>
	<li>
		is a collaborative work by several authors,
	</li>
	<li>
		there are other reasons that might make you miss your wiki.
	</li>
</ul>

<p>
	You might also consider hiding the site from public by making it
	<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-private')">private</a>;
</p>

{if isset($allowed)}
	<p>
		If you really really have a good reason to delete this wiki, click the button below:
	</p>

	<div id="sm-delete-box">
		<form>
			<div class="buttons">
				<input type="button" value="I want to delete this site"
				onclick="Wikijump.modules.ManagerSiteDeleteModule.listeners.deleteSite(event)"/>
			</div>
		</form>
	</div>
{else}
	<div class="error-block">
		Sorry, this option is available only to the founder of this site - {printuser user=$founder image=true}
	</div>
{/if}
