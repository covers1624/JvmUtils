/**
 * This file is derived from JavaPopExtractGenerator of covers1624/JdkUtils.
 * <p>
 * This file is licensed under the MIT license.
 * <p>
 * Generates a class which echos all the provided program arguments as system property Key-value pairs
 * to the program's standard output. Compatible with Java 1, roughly equivalent to the following Java:
 * <pre>
 *     public static void main(String[] var0) {
 *         PrintStream var1 = System.out;
 *
 *         for(int var2 = 0; var2 < var0.length; ++var2) {
 *             var1.print(var0[var2]);
 *             var1.print("=");
 *             var1.println(System.getProperty(var0[var2], ""));
 *         }
 *     }
 * </pre>
 * When invoked with the following arguments <code>"java.version" "java.vendor"</code> the following
 * standard output is generated:
 * <pre>
 *     java.version=1.8.0_292
 *     java.vendor=Oracle Corporation
 * </pre>
 * <p>
 * Usage: `groovy PropExtractGen.groovy <output_dir>`
 */
@Grab('org.ow2.asm:asm:9.2')
import org.objectweb.asm.ClassWriter
import org.objectweb.asm.Label
import org.objectweb.asm.MethodVisitor

import java.nio.file.Files
import java.nio.file.Path

import static org.objectweb.asm.Opcodes.*

if (args.length != 1) {
    println("Expected single argument.")
    System.exit(1)
}

ClassWriter cw = new ClassWriter(0)
cw.visit(V1_1, ACC_PUBLIC | ACC_SUPER, "PropExtract", null, "java/lang/Object", null)
MethodVisitor mv

mv = cw.visitMethod(ACC_PUBLIC, "<init>", "()V", null, null)
mv.visitCode()
mv.visitVarInsn(ALOAD, 0)
mv.visitMethodInsn(INVOKESPECIAL, "java/lang/Object", "<init>", "()V", false)
mv.visitInsn(RETURN)
mv.visitMaxs(1, 1)
mv.visitEnd()

mv = cw.visitMethod(ACC_PUBLIC | ACC_STATIC, "main", "([Ljava/lang/String;)V", null, null)
mv.visitCode();

int array = 0
int out = 1
int index = 2

mv.visitFieldInsn(GETSTATIC, "java/lang/System", "out", "Ljava/io/PrintStream;")
mv.visitVarInsn(ASTORE, out)

Label head = new Label()
Label after = new Label()
mv.visitInsn(ICONST_0)
mv.visitVarInsn(ISTORE, index)
mv.visitLabel(head)
mv.visitVarInsn(ILOAD, index)
mv.visitVarInsn(ALOAD, array)
mv.visitInsn(ARRAYLENGTH)
mv.visitJumpInsn(IF_ICMPGE, after)

mv.visitVarInsn(ALOAD, out)
mv.visitVarInsn(ALOAD, array)
mv.visitVarInsn(ILOAD, index)
mv.visitInsn(AALOAD)
mv.visitMethodInsn(INVOKEVIRTUAL, "java/io/PrintStream", "print", "(Ljava/lang/String;)V", false)

mv.visitVarInsn(ALOAD, out)
mv.visitLdcInsn("=")
mv.visitMethodInsn(INVOKEVIRTUAL, "java/io/PrintStream", "print", "(Ljava/lang/String;)V", false)

mv.visitVarInsn(ALOAD, out)
mv.visitVarInsn(ALOAD, array)
mv.visitVarInsn(ILOAD, index)
mv.visitInsn(AALOAD)
mv.visitLdcInsn("")
mv.visitMethodInsn(INVOKESTATIC, "java/lang/System", "getProperty", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;", false)
mv.visitMethodInsn(INVOKEVIRTUAL, "java/io/PrintStream", "println", "(Ljava/lang/String;)V", false)
mv.visitIincInsn(index, 1)
mv.visitJumpInsn(GOTO, head)

mv.visitLabel(after)
mv.visitInsn(RETURN)
mv.visitMaxs(4, 3)
mv.visitEnd()

Files.write(Path.of(args[0]).resolve("PropExtract.class"), cw.toByteArray())
