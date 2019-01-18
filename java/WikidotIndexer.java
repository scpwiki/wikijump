import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStreamReader;
import java.io.RandomAccessFile;
import java.io.StringReader;
import java.io.RandomAccessFile;
import java.nio.channels.FileLock;

import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.Document;
import org.apache.lucene.document.Field;
import org.apache.lucene.index.IndexModifier;
import org.apache.lucene.index.Term;
import org.apache.lucene.queryParser.QueryParser;
import org.apache.lucene.search.Hits;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;

public class WikidotIndexer {
	public static void main(String[] args) {
		try {
//process
			if (args.length == 4 && args[0].equals("process")) {

				// lock first
				FileLock lock = new RandomAccessFile(new File(args[3]), "rw").getChannel().tryLock();
				if (lock == null) {
					return;
				}
				
				IndexModifier im = new IndexModifier(args[1], new StandardAnalyzer(), false);
				BufferedReader qr = new BufferedReader(new InputStreamReader(new FileInputStream(new File(args[2]))));
				// read the whole file
				StringBuilder queue = new StringBuilder();
				while (true) {
					String line = qr.readLine();
					if (line == null) break;
					queue.append(line);
					queue.append("\n");
				}
				qr = new BufferedReader(new StringReader(queue.toString()));
				RandomAccessFile raf = new RandomAccessFile(args[2], "rw");
				raf.setLength(0);
				raf.close();
				lock.release();
				
				try {
					while (true) {
						
						try {
							String line = qr.readLine();
							
							if (line == null) { // EOF
								break;
							}
							
							String cmd = line.split(" ")[0];
							String id = line.split(" ")[1];
							
							if (cmd.equals("DELETE_PAGE")) {
								im.deleteDocuments(new Term("page_id", id));
							} else if (cmd.equals("DELETE_THREAD")) {
								im.deleteDocuments(new Term("thread_id", id));
							} else if (cmd.equals("DELETE_SITE")) {
								im.deleteDocuments(new Term("site_id", id));
							} else if (cmd.equals("INDEX_FTS")) {
	
								im.deleteDocuments(new Term("fts_id", id));
								Document doc = new Document();
								doc.add(new Field("fts_id", id, Field.Store.YES, Field.Index.TOKENIZED));
								
								while (true) {
									line = qr.readLine();
									if (line.trim().equals("")) { // empty line
										break;
									}
									
									args = line.split(" ", 4);
									String fieldType = args[0];
									String key = args[1];
									float boost = new Float(args[2]).floatValue();
									String value = args[3];
									
									Field field = new Field(key, value,	fieldType.equals("TEXT") ? Field.Store.YES : Field.Store.NO, Field.Index.TOKENIZED);
									field.setBoost(boost);
									doc.add(field);
									//System.out.println(key + ": " + value + " (" + boost + ")");
								}
								im.addDocument(doc);
							}
						} catch (ArrayIndexOutOfBoundsException e) { // file corrupted somehow
						}
					}
					im.optimize();
					im.close();
					
				} catch (Exception e) {
					im.close();
					throw e;
				}
			
			} else if (args.length == 3 && args[0].equals("search")) {
// search
				IndexSearcher indexSearcher = new IndexSearcher(args[1]);
				QueryParser qp = new QueryParser("content",	new StandardAnalyzer());
				Query query = qp.parse(args[2]);

				Hits hits = indexSearcher.search(query);
				int hitCount = hits.length();
				
				for (int i = 0; i < hitCount; i++) {
					Document doc = hits.doc(i);
					System.out.println(doc.get("fts_id"));
				}

			} else {
// invocation
				System.err.println("Usage:");
				System.err.println("  java -jar wikidotIndexer.jar process <index_dir> <queue_file> <lock_file>");
				System.err.println("  java -jar wikidotIndexer.jar search <index_dir> <search_phrase>");
				
			}
			
		} catch (Exception e) {
			System.err.println("Exception: " + e.getMessage());
			e.printStackTrace();
		}
	}
}
