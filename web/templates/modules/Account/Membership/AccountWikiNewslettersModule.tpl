<h1>Wiki newsletters subscription</h1>

<p>
	Administrators of the Wiki Sites might decide to send from time to time email newsletters.
	This is a nice thing to keep a community together. An obvious thing is that not everyone
	might wish to receive such newsletters - so here you go.
</p>

<h2>Default policy</h2>

<p>
	When you join a new Wiki - do you want to receive related newsletter? This describes
	the default behaviour. Most users want to keep this option enabled.
</p>

<form>
	<table class="form">
		<tr>
			<td>
				{t}Receive newsletters by default?{/t}
			</td>
			<td>
				<input type="checkbox" class="checkbox"
					id="sm-receive-newsletters-default"
					{if $defaultNewsletter == true}checked="checked"{/if}/>
			</td>
			<td>
				<input type="button" class="button" value="apply" />
			</td>
		</tr>
	</table>
</form>

<h2>Particular Wikis</h2>

<p>
	You can choose to receive or not newsletters from the Wikis you have joined.
</p>

{if isset($mems)}
	<form id="receive-wiki-newsletters-form">
		<table class="form grid">
			<tr>
				<th>
					Wiki
				</th>
				<th>
					{t}receive newsletters?{/t}
				</th>
			</tr>
			{foreach from=$mems item=mem}
				{assign var=site value=$mem->getSite()}
				<tr>
					<td>
						<a href="{$HTTP_SCHEMA}://{$site->getDomain()}">{$site->getName()|escape}</a>
						{if $site->getSubtitle()}
							<br/>
							{$site->getSubtitle()|escape}
						{/if}
					</td>
					<td>
						<input type="checkbox" class="checkbox receive-newsletter"
							{if $mem->getAllowNewsletter()}checked="checked"{/if}
						 />
					</td>
				</tr>
			{/foreach}
			<tr>
				<td colspan="2" style="text-align: right">
					<a href="javascript:;" onclick="Wikijump.modules.ASWikiNewslettersModule.listeners.checkAll(event, true)">check all</a>
					| <a href="javascript:;" onclick="Wikijump.modules.ASWikiNewslettersModule.listeners.checkAll(event, false)">uncheck all</a>
				</td>
			</tr>
		</table>

		<div class="buttons">
			<input type="button" value="cancel" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-settings')"/>
			<input type="button" value="save" />
		</div>
	</form>
{else}
	<p>
		You do not belong to any Wiki (yet).
	</p>
{/if}
