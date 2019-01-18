<h1>{t}Page Tags{/t}</h1>

{ltext lang="en"}
<p>
	Tags are a nice way to organize content in your Site. You can apply multiple tags (labels) to each
	of your pages. You can learn more what a tag is reading Wikipedia entries for  
	<a href="http://en.wikipedia.org/wiki/Tags" target="_blank">Tags</a> and
	<a href="http://en.wikipedia.org/wiki/Tag_cloud" target="_blank">Tag cloud</a>.
</p>
{/ltext}
{ltext lang="pl"}
<p>
	Tagi (etykiety) to dobry sposób na kategoryzowanie informacji na sajcie.
	Do każdej ze stron można dołączyć kilka tagów.
</p>
{/ltext}

<div class="error-block" id="page-tags-errors" style="display: none"></div>
<form action="dummy" onsubmit="WIKIDOT.modules.PageTagsModule.listeners.save(event)">
	<table class="form">
		<tr>
			<td>
				{t}Tags{/t}:
			</td>
			<td>
				<input type="text" class="text" size="50" id="page-tags-input" value="{$tags|escape}"/>
				<div class="sub">
					{t}Space-separated list of tags.{/t}
				</div>
			</td>
		</tr>
	</table>
</form>
<div class="buttons">
	<input type="button" value="{t}close{/t}" onclick="WIKIDOT.page.listeners.closeActionArea(event)"/>
	<input type="button" value="{t}clear{/t}" onclick="$('page-tags-input').value=''"/>
	<input type="button" value="{t}save tags{/t}" onclick="WIKIDOT.modules.PageTagsModule.listeners.save(event)"/>
</div>