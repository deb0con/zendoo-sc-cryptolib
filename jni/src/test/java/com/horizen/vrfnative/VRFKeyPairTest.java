package com.horizen.vrfnative;

import com.horizen.librustsidechains.FieldElement;
import org.junit.Test;

import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.assertEquals;

public class VRFKeyPairTest {

    @Test
    public void testGenerate() throws Exception {

        try(VRFKeyPair keyPair = VRFKeyPair.generate())
        {
            assertNotNull("Key pair generation was unsuccessful.", keyPair);

            assertTrue("Public key verification failed.", keyPair.getPublicKey().verifyKey());
        }
    }

    @Test
    public void testProveVerify() throws Exception {

        byte[] skBytes = {
            46, -9, -92, 88, 17, -102, 67, 58, 29, -128, 59, -63, -127, 27, 104, -39, 95, -77, -17, -108, -90, -95, -81,
            -104, -123, 117, 2, 54, 93, -59, 100, 27
        };

        byte[] messageBytes = {
            124, 61, 95, 121, -73, 61, -94, 28, 97, 93, -82, 116, 113, -93, -86, -100, -124, 9, 75, -85, -27, -41, -124,
            59, 101, -111, -88, -68, -62, -48, 99, 46
        };

        byte[] proofBytes = {
            120, -24, -44, 5, -111, -17, -71, -9, -118, -89, -113, -28, 70, 126, -105, 74, 76, -120, 79, 31, 1, -60, 24,
            57, -68, 23, -21, 124, -18, 86, 17, 59, 0, 5, 78, -75, -69, 104, 112, 108, 89, 124, 102, -77, 71, -96, 119,
            62, 113, -87, 66, 89, -63, 20, -128, -91, -85, 76, 89, 57, 5, -94, -60, -60, 26, 16, 3, 87, -73, 31, -32, 74,
            72, 44, -91, 124, 68, -106, 102, 43, 50, -103, -93, -79, 34, 57, 71, -1, 5, 113, -7, -60, -84, 72, 10, 105, 50
        };

        byte[] vrfOutputBytes = {
            -88, 45, 7, 66, -66, 34, -62, 121, -44, 59, 33, -16, 88, 10, 13, -103, 112, -121, 78, -94, 12, -122, -73, 58,
            -109, 49, 18, -116, 33, -126, 33, 50
        };

        try
        (
            VRFSecretKey sk = VRFSecretKey.deserialize(skBytes);
            VRFKeyPair keyPair = new VRFKeyPair(sk);
            FieldElement message = FieldElement.deserialize(messageBytes);
            VRFProof proof = VRFProof.deserialize(proofBytes);
            FieldElement expectedVrfOutput = FieldElement.deserialize(vrfOutputBytes)
        )
        {
            assertNotNull("sk deserialization must not fail", sk);
            assertNotNull("message deserialization must not fail", message);
            assertNotNull("proof deserialization must not fail", proof);
            assertNotNull("expectedVrfOutput deserialization must not fail", sk);

            try(FieldElement vrfOutput = keyPair.getPublicKey().proofToHash(proof, message))
            {
                assertNotNull("VRF Proof verification and VRF Output computation has failed.", vrfOutput);

                // Check vrfOutput == expectedVrfOutput
                assertEquals("vrfOutput and expectedVrfOutput must be equal", vrfOutput, expectedVrfOutput);
            }
        }
    }

    @Test
    public void testRandomProveVerify() throws Exception {
        int samples = 100;

        for(int i = 0; i < samples; i++) {
            try
            (
                VRFKeyPair keyPair = VRFKeyPair.generate();
                FieldElement fieldElement = FieldElement.createRandom();
                FieldElement wrongFieldElement = FieldElement.createRandom()
            )
            {
                assertNotNull("Key pair generation was unsuccessful.", keyPair);
                assertTrue("Public key verification failed.", keyPair.getPublicKey().verifyKey());

                try
                (
                    VRFProveResult proofVRFOutputPair = keyPair.prove(fieldElement);
                    FieldElement vrfOutput = keyPair.getPublicKey().proofToHash(proofVRFOutputPair.getVRFProof(), fieldElement)
                )
                {
                    assertNotNull("Attempt to create vrf proof and output failed.", proofVRFOutputPair);
                    assertNotNull("VRF Proof verification and VRF Output computation must not fail.", vrfOutput);
                    assertEquals("prove() and proof_to_hash() vrf outputs must be equal", proofVRFOutputPair.getVRFOutput(), vrfOutput);
                    assertNull("VRF Proof verification must fail", keyPair.getPublicKey().proofToHash(proofVRFOutputPair.getVRFProof(), wrongFieldElement));
                }
            }
        }
    }
}
