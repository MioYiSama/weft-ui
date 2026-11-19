package dev.weft

import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

class CorpusTest {
    private val corpus = File(System.getProperty("weft.corpus"))

    @Test
    fun decodesOpsAndErrors() {
        load("ops.json").getJSONArray("cases").forEachObject { case ->
            val name = case.getString("name")
            val bytes = ByteBuffer.wrap(case.bytes())
            if (case.has("error")) {
                val error = assertThrows(name, OpException::class.java) { OpReader.readAll(bytes) }
                assertEquals(name, case.getString("error"), error.kind)
            } else {
                val ops = case.getJSONArray("ops")
                assertEquals(name, List(ops.length()) { op(ops.getJSONObject(it)) }, OpReader.readAll(bytes))
            }
        }
    }

    @Test
    fun encodesValues() {
        load("values.json").getJSONArray("cases").forEachObject { case ->
            assertArrayEquals(case.getString("name"), case.bytes(), value(case.getJSONObject("value")).encode())
        }
    }

    @Test
    fun handlesFrames() {
        val frames = load("frames.json")
        frames.getJSONArray("requests").forEachObject { case ->
            val (head, body) = PlatformModules.unframe(case.bytes())
            assertFrame(case, head, body)
        }
        frames.getJSONArray("responses").forEachObject { case ->
            val bytes = PlatformModules.frame(case.getJSONObject("head"), case.getString("body").encodeToByteArray())
            val buffer = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
            val length = buffer.int.toUInt().toLong()
            assertTrue("${case.getString("name")}: head length $length", length <= buffer.remaining())
            val head = ByteArray(length.toInt()).also { buffer.get(it) }
            val body = ByteArray(buffer.remaining()).also { buffer.get(it) }
            assertFrame(case, JSONObject(head.decodeToString()), body)
        }
        frames.getJSONArray("malformed").forEachObject { case ->
            assertThrows(case.getString("name"), PlatformFailure::class.java) { PlatformModules.unframe(case.bytes()) }
        }
    }

    private fun load(file: String): JSONObject = JSONObject(corpus.resolve(file).readText())

    private fun op(json: JSONObject): Op {
        val scope = json.getInt("scope")
        return when (val name = json.getString("op")) {
            "create" -> Op.Create(scope, json.getInt("template"))
            "set" -> Op.Set(scope, json.getInt("slot"), value(json.getJSONObject("value")))
            "branch" -> Op.Branch(scope, json.getInt("point"), json.getInt("child"))
            "rows" -> {
                val children = json.getJSONArray("children")
                Op.Rows(scope, json.getInt("point"), IntArray(children.length()) { children.getInt(it) })
            }
            "attach" -> Op.Attach(scope, json.getInt("point"), json.getInt("child"))
            else -> error("Unknown corpus op: $name")
        }
    }

    private fun value(json: JSONObject): WeftValue = when {
        json.has("bool") -> WeftValue.Bool(json.getBoolean("bool"))
        json.has("number") -> WeftValue.Number(json.getDouble("number"))
        json.has("string") -> WeftValue.Text(json.getString("string"))
        else -> error("Unknown corpus value: $json")
    }

    private fun assertFrame(case: JSONObject, head: JSONObject, body: ByteArray) {
        val name = case.getString("name")
        assertEquals(name, case.getJSONObject("head").jsonValue(), head.jsonValue())
        assertArrayEquals(name, case.getString("body").encodeToByteArray(), body)
    }

    private fun Any?.jsonValue(): Any? = when (this) {
        is JSONObject -> keys().asSequence().associateWith { get(it).jsonValue() }
        is JSONArray -> List(length()) { get(it).jsonValue() }
        is Number -> toString().toBigDecimal().stripTrailingZeros()
        JSONObject.NULL -> null
        else -> this
    }

    private fun JSONObject.bytes(): ByteArray = getString("hex").filterNot(Char::isWhitespace).hexToByteArray()

    private inline fun JSONArray.forEachObject(action: (JSONObject) -> Unit) {
        for (index in 0 until length()) action(getJSONObject(index))
    }
}
